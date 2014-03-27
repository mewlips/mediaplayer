use avcodec;
use avformat;
use avutil;
use av_stream::{AVStream,AVStreamIterator};
use std::ptr::mut_null;
use util;
use std::mem::size_of;
use std::cast::{transmute};
use std::libc::c_int;
use component::{Component,ComponentStruct,ExtractorComponent,
                VideoDecoderComponent,AudioDecoderComponent,ManagerComponent};
use message::{Message,MsgStop,MsgSeek,MsgFlush,
              MsgExtract,MsgError,MsgEOF,MsgPacketData};

pub struct Extractor {
    component: Option<ComponentStruct>,
    priv fmt_ctx: *mut avformat::AVFormatContext,
    streams: Vec<AVStream>,
    video_index: Option<int>,
    audio_index: Option<int>,
    video_time_base: Option<avutil::AVRational>,
    audio_time_base: Option<avutil::AVRational>,
}

impl Extractor {
    pub fn new(path: &Path) -> Option<Extractor> {
        let mut extractor = Extractor {
            component: Some(ComponentStruct::new(ExtractorComponent)),
            fmt_ctx: unsafe { avformat::avformat_alloc_context() },
            streams: vec!(),
            video_index: None,
            audio_index: None,
            video_time_base: None,
            audio_time_base: None,
        };

        if extractor.fmt_ctx.is_null() {
            return None;
        }

        let mut result = path.with_c_str(|path| {
            unsafe { 
                avformat::avformat_open_input(&mut extractor.fmt_ctx, path,
                                              mut_null(), mut_null())
            }
        });
        if result < 0 {
            error!("avformat_open_input() failed! {}", util::av_strerror(result));
            return None;
        }

        result = unsafe {
            avformat::avformat_find_stream_info(extractor.fmt_ctx, mut_null())
        };
        if result < 0 {
            error!("avformat_find_stream_info() failed! {}", util::av_strerror(result));
            return None;
        }

        path.with_c_str(|path| {
            unsafe {
                avformat::av_dump_format(extractor.fmt_ctx, 0, path, 0)
            }
        });

        for av_stream in extractor.iter() {
            match av_stream.get_type() {
                avutil::AVMEDIA_TYPE_AUDIO => info!("audio stream found"),
                avutil::AVMEDIA_TYPE_VIDEO => info!("video stream found"),
                type_ => info!("stream found (type = {})", type_)
            }
            extractor.streams.push(av_stream);
        }

        Some(extractor)
    }
    pub fn iter(&self) -> AVStreamIterator {
        unsafe {
            AVStreamIterator {
                nb_streams: (*self.fmt_ctx).nb_streams,
                offset: 0,
                streams: (*self.fmt_ctx).streams
            }
        }
    }
    pub fn get_stream<'r>(&'r mut self, type_: avutil::Enum_AVMediaType, index: int)
            -> Option<&'r AVStream> {
        let mut count = index;
        for av_stream in self.streams.iter() {
            if av_stream.get_type() == type_
               && count >= 0 {
                if count == 0 {
                    if type_ == avutil::AVMEDIA_TYPE_VIDEO {
                        self.video_index = Some(av_stream.get_index());
                        self.video_time_base = Some(av_stream.get_time_base());
                    } else if type_ == avutil::AVMEDIA_TYPE_AUDIO {
                        self.audio_index = Some(av_stream.get_index());
                        self.audio_time_base = Some(av_stream.get_time_base());
                    }
                    return Some(av_stream);
                } else {
                    count -= 1;
                }
            }
        }
        None
    }
    pub fn start(&mut self) {
        let fmt_ctx = self.fmt_ctx.clone();
        let video_index = self.video_index.clone();
        let audio_index = self.audio_index.clone();
        let video_time_base = self.video_time_base.clone();
        let audio_time_base = self.audio_time_base.clone();
        let component = self.component.take().unwrap();
        spawn(proc() {
            component.wait_for_start();
            let mut stopped = false;
            while Extractor::pump(&component, fmt_ctx,
                                  video_index, audio_index) {
                match component.recv() {
                    Message { msg: MsgExtract, .. } => {
                        //debug!("MsgExtract");
                    }
                    Message { msg: MsgStop, .. } => {
                        component.flush();
                        info!("stop Extractor");
                        stopped = true;
                        break;
                    }
                    Message { msg: MsgSeek(pts, flag), .. } => {
                        let seek_pos = (pts * avutil::AV_TIME_BASE as f64) as i64;
                        Extractor::seek(&component, fmt_ctx, seek_pos, flag,
                                        video_index, audio_index,
                                        video_time_base, audio_time_base);
                    }
                    _ => {
                        error!("unexpected message received");
                        break;
                    }
                }
            }
            if !stopped { loop {
                match component.recv() {
                    Message { msg: MsgExtract, .. } => {
                        debug!("ignore MsgExtract");
                    }
                    Message { msg: MsgStop, .. } => {
                        info!("stop Extractor");
                        break;
                    }
                    _ => {
                        error!("unexpected message received");
                        break;
                    }
                }
            }}
        })
    }
    fn pump(component: &ComponentStruct,
            fmt_ctx: *mut avformat::AVFormatContext,
            video_index: Option<int>, audio_index: Option<int>) -> bool {
        let size = size_of::<avcodec::AVPacket>();
        let packet: *mut avcodec::AVPacket = unsafe {
            transmute(avutil::av_malloc(size as u64))
        };
        if packet.is_null() {
            component.send(ManagerComponent, MsgError(~"Allocation failed"));
            return false;
        }

        let result = unsafe {
            avcodec::av_init_packet(packet);
            avformat::av_read_frame(fmt_ctx, packet)
        };
        if result >= 0 {
            let stream_index = unsafe {
                (*packet).stream_index as int
            };
            match video_index {
                Some(video_index) => {
                    if video_index == stream_index {
                        component.send(VideoDecoderComponent, MsgPacketData(packet));
                    }
                }
                None => {
                }
            }
            match audio_index {
                Some(audio_index) => {
                    if audio_index == stream_index {
                        component.send(AudioDecoderComponent, MsgPacketData(packet));
                    }
                }
                None => {
                }
            }
            return true;
        } else {
            info!("end of file");
            component.send(ManagerComponent, MsgEOF);
            return false;
        }
    }
    fn seek(component: &ComponentStruct,
            fmt_ctx: *mut avformat::AVFormatContext,
            seek_pos: i64, flags: c_int,
            video_index: Option<int>, audio_index: Option<int>,
            video_time_base: Option<avutil::AVRational>,
            audio_time_base: Option<avutil::AVRational>) -> bool {
        //debug!("seek(), pos = {}, flags = {}", seek_pos, flags);
        let mut stream_index;
        let seek_target = unsafe {
            if video_index.is_some() {
                stream_index = video_index.unwrap();
                avutil::av_rescale_q(seek_pos, avutil::AV_TIME_BASE_Q,
                                     video_time_base.unwrap())
            } else /*audio_index.is_some()*/ {
                stream_index = audio_index.unwrap();
                avutil::av_rescale_q(seek_pos, avutil::AV_TIME_BASE_Q,
                                     audio_time_base.unwrap())
            }
        };
        //debug!("seek(), seek_target = {}, index = {}", seek_target, stream_index);
        let result = unsafe {
            avformat::av_seek_frame(
                fmt_ctx, stream_index as i32, seek_target, flags)
        };
        if result < 0 {
            error!("seek failed");
            return false;
        }

        component.send(VideoDecoderComponent, MsgFlush);
        component.send(AudioDecoderComponent, MsgFlush);

        true
    }
}

impl Drop for Extractor {
    fn drop(&mut self) {
        debug!("Extractor::drop()");
    }
}

impl Component for Extractor {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct {
        self.component.get_mut_ref()
    }
}
