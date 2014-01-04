use avcodec;
use avformat;
use avutil;
use av_stream::{AVStream,AVStreamIterator};
use std::ptr::mut_null;
use util;
use std::mem::size_of;
use std::cast::{transmute};
use component_manager::{Component,ComponentStruct,ExtractorComponent,
                        VideoDecoderComponent,AudioDecoderComponent,
                        ManagerComponent,Message,MsgStart,MsgStop,
                        MsgExtract,MsgError,MsgEOF,MsgPacketData};

pub struct Extractor {
    component: Option<ComponentStruct>,
    priv fmt_ctx: *mut avformat::AVFormatContext,
    streams: ~[AVStream],
    video_index: Option<int>,
    audio_index: Option<int>,
}

impl Extractor {
    pub fn new(path: &Path) -> Option<Extractor> {
        let mut extractor = Extractor {
            component: Some(ComponentStruct::new(ExtractorComponent)),
            fmt_ctx: unsafe { avformat::avformat_alloc_context() },
            streams: ~[],
            video_index: None,
            audio_index: None,
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
            extractor.streams = extractor.streams + ~[av_stream];
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
                    } else if type_ == avutil::AVMEDIA_TYPE_AUDIO {
                        self.audio_index = Some(av_stream.get_index());
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
        let component = self.component.take().unwrap();
        do spawn {
            match component.recv() {
                Message { from: ManagerComponent, msg: MsgStart, .. } => {
                    info!("start Extractor");
                }
                _ => {
                    fail!("unexpected message received");
                }
            }
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
        }
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
