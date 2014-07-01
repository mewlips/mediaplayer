use avcodec;
use av_stream::AVStream;
use avutil;
use std::ptr::{mut_null};
use std::mem::{transmute};
use util;
use std::mem::size_of;

pub struct DecoderUserData {
    pub pts: u64,
}

pub struct FFmpegDecoder {
    pub codec_ctx: *mut avcodec::AVCodecContext,
    pub codec: *const avcodec::AVCodec,
    pub decoder: *mut avcodec::AVCodec,
    pub options: *mut avutil::AVDictionary,
    pub time_base: avutil::AVRational
}

impl FFmpegDecoder {
    pub fn new(av_stream: &AVStream) -> Option<FFmpegDecoder> {
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
            avcodec::avcodec_open2(codec_ctx, &*decoder, &mut options)
        };
        if result < 0 {
            error!("AVError: {}", util::av_strerror(result));
            return None;
        }
        unsafe {
            (*codec_ctx).opaque = avutil::av_malloc(size_of::<DecoderUserData>() as u64);
        }

        let time_base = av_stream.get_time_base();

        Some(FFmpegDecoder {
            codec_ctx: codec_ctx,
            codec: codec,
            decoder: decoder,
            options: options,
            time_base: time_base,
        })
    }
}

impl Drop for FFmpegDecoder {
    fn drop(&mut self) {
        println!("FFmpegDecoder::drop()");
        unsafe {
            if !(*self.codec_ctx).opaque.is_null() {
                avutil::av_freep(transmute(&mut (*self.codec_ctx).opaque));
            }
        }
    }
}
