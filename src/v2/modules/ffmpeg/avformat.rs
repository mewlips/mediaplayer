use libc::types::os::arch::c95::{c_int};
use ll_avformat;
use modules::ffmpeg::avcodec::AVPacket;
use modules::ffmpeg::avutil::AVDictionary;
use modules::ffmpeg::result::{AvResult,AvError};
use std::ptr::mut_null;
use std::mem::transmute;
use std::str::SendStr;

pub fn av_register_all() {
    unsafe {
        ll_avformat::av_register_all();
    }
}

pub struct AVFormatContext {
    pub raw: *mut ll_avformat::AVFormatContext
}

impl AVFormatContext {
    pub fn alloc_context() -> AVFormatContext {
        AVFormatContext {
            raw: unsafe { ll_avformat::avformat_alloc_context() }
        }
    }

    pub fn open_input(&mut self, path: &Path) -> AvResult<()> {
        let result = path.with_c_str(|path| {
            unsafe {
                ll_avformat::avformat_open_input(&mut self.raw, path, mut_null(), mut_null())
            }
        });
        if result == 0 {
            Ok(())
        } else {
            Err(AvError::new(format!("failed to open '{}'", path.display()), result))
        }
    }

    pub fn find_stream_info(&mut self, opt: Option<AVDictionary>)
            -> AvResult<Option<AVDictionary>> {
        let result = unsafe { ll_avformat::avformat_find_stream_info(
            self.raw,
            opt.map_or(mut_null(), |dict| { transmute(&dict.raw) })
        )};

        match result {
            0 => {
                match opt {
                    a @ Some(_) => Ok(a),
                    None => Ok(None)
                }
            }
            e => {
                Err(AvError::new("find_stream_info() failed", result))
            }
        }
    }
    pub fn dump_format(&mut self, index: int, url: &Path, is_output: bool) {
        url.with_c_str(|url| {
            unsafe {
                ll_avformat::av_dump_format(self.raw, index as c_int, url,
                                            if is_output { 1 } else { 0 })
            }
        });
    }
    pub fn read_frame(&mut self, packet: &mut AVPacket) -> AvResult<()> {
        let result = unsafe {
            ll_avformat::av_read_frame(self.get_raw_ref(), packet.get_raw_ref())
        };
        if result == 0 {
            Ok(())
        } else {
            Err(AvError::new("av_read_frame() failed", result))
        }
    }
    pub fn get_raw_ref(&mut self) -> &mut ll_avformat::AVFormatContext {
        unsafe {
            &mut (*self.raw)
        }
    }
}

pub struct AVStream {
    raw: *mut ll_avformat::AVStream
}

impl AVStream {
}
