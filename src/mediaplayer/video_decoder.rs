use avcodec;
use av_stream::AVStream;
use avutil;
use std::cast::{transmute,transmute_immut_unsafe};
use std::libc::{c_int};
use std::ptr::{to_mut_unsafe_ptr};
use ffmpeg_decoder::{DecoderUserData,FFmpegDecoder};
use std::mem::size_of;
use component_manager::{Component,ComponentId,Message};

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
    component_id: Option<ComponentId>,
    chan: Option<SharedChan<Message>>,
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
                    component_id: None,
                    chan: None,
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
    pub fn start(&self, vd_port: Port<Option<*mut avcodec::AVPacket>>,
                        vr_chan: Chan<Option<~VideoData>>) {
        let codec_ctx = self.decoder.codec_ctx.clone();
        let time_base = self.decoder.time_base.clone();
        do spawn {
            while VideoDecoder::decode(codec_ctx, time_base,
                                       &vd_port, &vr_chan) {
                ;
            }
        }
    }
    fn decode(codec_ctx: *mut avcodec::AVCodecContext,
              time_base: avutil::AVRational,
              vd_port: &Port<Option<*mut avcodec::AVPacket>>,
              vr_chan: &Chan<Option<~VideoData>>) -> bool {
        match vd_port.recv() {
            Some(packet) => {
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
                        println!("use pts");
                    } else if (*packet).dts != avutil::AV_NOPTS_VALUE {
                        println!("use dts");
                        pts = (*packet).dts as f64;
                    } else {
                        pts = 0f64;
                    }
                    pts = ((pts as f64) * avutil::av_q2d(time_base)) as f64;
                    //println!("pts = {}", pts);
                    avcodec::av_free_packet(packet);
                    //println!("pts = {}, dts = {}", (*packet).pts, (*packet).dts);
                    if got_frame != 0 {
                        //debug!("send frame = {}", frame);
                        vr_chan.send(Some(~VideoData::new(frame,pts)));
                    }
                }
                true
            }
            None => {
                info!("null packet received");
                vr_chan.send(None);
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
    fn set_id(&mut self, id: ComponentId) {
        self.component_id = Some(id);
    }
    fn get_id(&self) -> Option<ComponentId> {
        self.component_id
    }
    fn get_name(&self) -> &str {
        "VideoDecoder"
    }
    fn set_chan(&mut self, chan: SharedChan<Message>) {
        self.chan = Some(chan);
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
        println!("get_buffer(): pts = {}", (*pts));
    }
    ret
}

extern fn release_buffer(codec_ctx: *mut avcodec::AVCodecContext,
                         pic: *mut avcodec::AVFrame) {
    unsafe {
        avcodec::avcodec_default_release_buffer(codec_ctx, pic);
    }
    println!("release_buffer()");
}
