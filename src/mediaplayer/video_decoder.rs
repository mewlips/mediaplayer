use avcodec;
use av_stream::AVStream;
use avutil;
use std::cast::{transmute_immut_unsafe};
use std::libc::{c_int};
use std::ptr::{to_mut_unsafe_ptr};
use ffmpeg_decoder::FFmpegDecoder;

struct VideoDecoder {
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
                Some(VideoDecoder {
                    decoder: decoder,
                    width: width,
                    height: height,
                    pix_fmt: pix_fmt
                })
            }
            None => {
                None
            }
        }
    }
    pub fn start(&self, vd_port: Port<Option<*mut avcodec::AVPacket>>,
                        vr_chan: Chan<Option<*mut avcodec::AVFrame>>) {
        let codec_ctx = self.decoder.codec_ctx.clone();
        do spawn {
            while VideoDecoder::decode(codec_ctx, &vd_port, &vr_chan) {
                ;
            }
        }
    }
    fn decode(codec_ctx: *mut avcodec::AVCodecContext,
              vd_port: &Port<Option<*mut avcodec::AVPacket>>,
              vr_chan: &Chan<Option<*mut avcodec::AVFrame>>) -> bool {
        match vd_port.recv() {
            Some(packet) => {
                unsafe {
                    let mut got_frame: c_int = 0;
                    let frame = avcodec::avcodec_alloc_frame();
                    avcodec::avcodec_decode_video2(codec_ctx, frame,
                                                   to_mut_unsafe_ptr(&mut got_frame),
                                                   transmute_immut_unsafe(packet));
                    avcodec::av_free_packet(packet);
                    if got_frame != 0 {
                        //debug!("send frame = {}", frame);
                        vr_chan.send(Some(frame));
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
