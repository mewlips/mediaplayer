use avcodec;
use av_stream::AVStream;
use avutil;
use std::cast::{transmute,transmute_immut_unsafe};
use std::libc::{c_int};
use std::ptr::{to_mut_unsafe_ptr};
use ffmpeg_decoder::{DecoderUserData,FFmpegDecoder};
use std::mem::size_of;
use component::{Component,ComponentStruct,VideoDecoderComponent,
                ClockComponent,ExtractorComponent,VideoRendererComponent};
use message::{Message,MsgPts,MsgStop,MsgFlush,
              MsgExtract,MsgPacketData,MsgVideoData};

#[deriving(Clone)]
pub struct VideoData {
    frame: *mut avcodec::AVFrame,
    pts: f64,
}

impl VideoData {
    pub fn new(frame: *mut avcodec::AVFrame, pts: f64) -> VideoData {
        VideoData {
            frame: frame,
            pts: pts
        }
    }
}

pub struct VideoDecoder {
    component: Option<ComponentStruct>,
    decoder: FFmpegDecoder,
    width: int,
    height: int,
    pix_fmt: avutil::Enum_AVPixelFormat,
}

impl VideoDecoder {
    pub fn new(video_stream: &AVStream) -> Option<VideoDecoder> {
        match FFmpegDecoder::new(video_stream) {
            Some(decoder) => {
                let codec_ctx = unsafe { *decoder.codec_ctx };
                let width = codec_ctx.width as int;
                let height = codec_ctx.height as int;
                let pix_fmt = codec_ctx.pix_fmt;
                unsafe {
                    (*decoder.codec_ctx).get_buffer = get_buffer;
                    (*decoder.codec_ctx).release_buffer = release_buffer;
                }
                Some(VideoDecoder {
                    component: Some(ComponentStruct::new(VideoDecoderComponent)),
                    decoder: decoder,
                    width: width,
                    height: height,
                    pix_fmt: pix_fmt,
                })
            }
            None => {
                None
            }
        }
    }
    pub fn start(&mut self) {
        let codec_ctx = self.decoder.codec_ctx.clone();
        let time_base = self.decoder.time_base.clone();
        let component = self.component.take().unwrap();
        spawn(proc() {
            component.wait_for_start();
            while VideoDecoder::decode(&component, codec_ctx, time_base) {
                ;
            }
            info!("stop VideoDecoder");
        })
    }
    fn decode(component: &ComponentStruct,
              codec_ctx: *mut avcodec::AVCodecContext,
              time_base: avutil::AVRational) -> bool {
        match component.recv() {
            Message { msg: MsgPacketData(packet), .. } => {
                unsafe {
                    let mut got_frame: c_int = 0;
                    let frame = avcodec::avcodec_alloc_frame();
                    let mut pts = 0f64;
                    let user_data: *mut DecoderUserData = transmute((*codec_ctx).opaque);
                    (*user_data).pts = (*packet).pts as u64;
                    avcodec::avcodec_decode_video2(codec_ctx, frame,
                                                   to_mut_unsafe_ptr(&mut got_frame),
                                                   transmute_immut_unsafe(packet));
                    if (*packet).dts == avutil::AV_NOPTS_VALUE &&
                       !(*frame).opaque.is_null() {
                        let frame_pts: *u64 = transmute((*frame).opaque);
                        if *frame_pts == avutil::AV_NOPTS_VALUE as u64 {
                            pts = (*packet).dts as f64;
                        }
                        //debug!("use pts");
                    } else if (*packet).dts != avutil::AV_NOPTS_VALUE {
                        //debug!("use dts");
                        pts = (*packet).dts as f64;
                    } else {
                        pts = 0f64;
                    }
                    pts = ((pts as f64) * avutil::av_q2d(time_base)) as f64;
                    //println!("pts = {}", pts);
                    avcodec::av_free_packet(packet);
                    //println!("pts = {}, dts = {}", (*packet).pts, (*packet).dts);
                    if got_frame != 0 {
                        component.send(ClockComponent, MsgPts(pts.clone()));
                        //debug!("send frame = {}", frame);
                        component.send(VideoRendererComponent,
                                       MsgVideoData(~VideoData::new(frame,pts)));
                    } else {
                        component.send(ExtractorComponent, MsgExtract);
                    }
                }
                true
            }
            Message { msg: MsgStop, .. } => {
                component.flush();
                false
            }
            Message { msg: MsgFlush, .. } => {
                component.flush();
                unsafe {
                    avcodec::avcodec_flush_buffers(codec_ctx);
                }
                true
            }
            _ => {
                error!("unexpected message recevied");
                false
            }
        }
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        debug!("VideoDecoder::drop()");
    }
}

impl Component for VideoDecoder {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct {
        self.component.get_mut_ref()
    }
}

extern fn get_buffer(codec_ctx: *mut avcodec::AVCodecContext,
                     pic: *mut avcodec::AVFrame) -> c_int {
    let ret = unsafe {
        avcodec::avcodec_default_get_buffer(codec_ctx, pic)
    };
    unsafe {
        let user_data: *mut DecoderUserData = transmute((*codec_ctx).opaque);
        let pts: *mut u64 = transmute(avutil::av_malloc(size_of::<u64>() as u64));
        (*pts) = (*user_data).pts;
        (*pic).opaque = transmute(pts);
        //debug!("get_buffer(): pts = {}", (*pts));
    }
    ret
}

extern fn release_buffer(codec_ctx: *mut avcodec::AVCodecContext,
                         pic: *mut avcodec::AVFrame) {
    unsafe {
        avcodec::avcodec_default_release_buffer(codec_ctx, pic);
    }
    //debug!("release_buffer()");
}
