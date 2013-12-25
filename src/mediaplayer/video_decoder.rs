use avcodec;
use av_stream::AVStream;
use avutil;
use std::cast::{transmute_immut_unsafe};
use std::libc::{c_int};
use std::ptr::{to_mut_unsafe_ptr};
use ffmpeg_decoder::FFmpegDecoder;

pub struct VideoDecoder {
    decoder: FFmpegDecoder,
    width: int,
    height: int,
    pix_fmt: avutil::Enum_AVPixelFormat,
    time_base: avutil::AVRational,
}

impl VideoDecoder {
    pub fn new(video_stream: &AVStream) -> Option<VideoDecoder> {
        match FFmpegDecoder::new(video_stream) {
            Some(decoder) => {
                let codec_ctx = unsafe { *decoder.codec_ctx };
                let width = codec_ctx.width as int;
                let height = codec_ctx.height as int;
                let pix_fmt = codec_ctx.pix_fmt;
                let time_base = video_stream.get_time_base();
                Some(VideoDecoder {
                    decoder: decoder,
                    width: width,
                    height: height,
                    pix_fmt: pix_fmt,
                    time_base: time_base
                })
            }
            None => {
                None
            }
        }
    }
    pub fn start(&self, vd_port: Port<Option<*mut avcodec::AVPacket>>,
                        vs_chan: Chan<Option<*mut avcodec::AVFrame>>) {
        let codec_ctx = self.decoder.codec_ctx.clone();
        let time_base = self.time_base;
        do spawn {
            while VideoDecoder::decode(codec_ctx, time_base, &vd_port, &vs_chan) {
                ;
            }
        }
    }
    fn decode(codec_ctx: *mut avcodec::AVCodecContext,
              time_base: avutil::AVRational,
              vd_port: &Port<Option<*mut avcodec::AVPacket>>,
              vs_chan: &Chan<Option<*mut avcodec::AVFrame>>) -> bool {
        match vd_port.recv() {
            Some(packet) => {
                unsafe {
                    let mut got_frame: c_int = 0;
                    let frame = avcodec::avcodec_alloc_frame();
                    let mut pts = 0f64;
                    avcodec::avcodec_decode_video2(codec_ctx, frame,
                                                   to_mut_unsafe_ptr(&mut got_frame),
                                                   transmute_immut_unsafe(packet));
                    if (*packet).dts != avutil::AV_NOPTS_VALUE {
                        pts = (*packet).dts as f64;
                    }
                    pts = ((pts as f64) * avutil::av_q2d(time_base)) as f64;
                    println!("pts = {}", pts);
                    avcodec::av_free_packet(packet);
                    //println!("pts = {}, dts = {}", (*packet).pts, (*packet).dts);
                    if got_frame != 0 {
                        //debug!("send frame = {}", frame);
                        vs_chan.send(Some(frame));
                    }
                }
                true
            }
            None => {
                info!("null packet received");
                vs_chan.send(None);
                false
            }
        }
    }
}
