use avcodec;
use av_stream::AVStream;
use avutil;
use std::ptr::{mut_null,to_mut_unsafe_ptr};
use std::cast::{transmute_immut_unsafe};
use util;

struct FFmpegDecoder {
    codec_ctx: *mut avcodec::AVCodecContext,
    codec: *avcodec::AVCodec,
    decoder: *mut avcodec::AVCodec,
    options: *mut avutil::AVDictionary,
}

impl FFmpegDecoder {
    pub fn new(av_stream: &AVStream) -> Option<FFmpegDecoder> {
        debug!("VideoDecoder::new() start");
        let codec_ctx = unsafe {
            (*av_stream.av_stream).codec
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
        Some(FFmpegDecoder {
            codec_ctx: codec_ctx,
            codec: codec,
            decoder: decoder,
            options: options,
        })
    }
}
