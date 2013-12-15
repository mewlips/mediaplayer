use avcodec;
use avstream::AVStream;
use avutil;
use std::cast::{transmute_immut_unsafe};
use std::libc::{c_int};
use std::ptr::{mut_null,to_mut_unsafe_ptr};
use util;

struct VideoDecoder {
    priv codec_ctx: *mut avcodec::AVCodecContext,
    priv codec: *avcodec::AVCodec,
    priv decoder: *mut avcodec::AVCodec,
    priv options: *mut avutil::AVDictionary,
}

impl VideoDecoder {
    pub fn new(stream_info: &AVStream)
            -> Option<VideoDecoder> {
        debug!("VideoDecoder::new() start");
        let codec_ctx = unsafe {
            (*stream_info.av_stream).codec
        };
        if codec_ctx.is_null() {
            debug!("codec not found");
            return None;
        }
        let codec = unsafe {
            (*codec_ctx).codec
        };
        let decoder = unsafe {
            avcodec::avcodec_find_decoder((*codec_ctx).codec_id)
        };
        if decoder.is_null() {
            error!("avcodec_find_decoder failed!");
            return None;
        }
        let mut options = mut_null();
        let result = unsafe {
            avcodec::avcodec_open2(codec_ctx, transmute_immut_unsafe(decoder),
                                   to_mut_unsafe_ptr(&mut options))
        };
        if result < 0 {
            error!("AVError: {}", util::av_strerror(result));
            return None;
        }
        debug!("VideoDecoder::new() end");
        Some(VideoDecoder {
            codec_ctx: codec_ctx,
            codec: codec,
            decoder: decoder,
            options: options,
        })
    }
    pub fn get_width(&self) -> int {
        unsafe {
            (*self.codec_ctx).width as int
        }
    }
    pub fn get_height(&self) -> int {
        unsafe {
            (*self.codec_ctx).height as int
        }
    }
    pub fn get_pix_fmt(&self) -> avutil::Enum_AVPixelFormat {
        unsafe {
            (*self.codec_ctx).pix_fmt
        }
    }
    pub fn start(&self, vd_port: Port<Option<*mut avcodec::AVPacket>>,
                        vr_chan: Chan<Option<*mut avcodec::AVFrame>>) {
        let codec_ctx = self.codec_ctx.clone();
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
