use avformat;
use avutil;
use std::libc::{c_uint};

pub struct AVStream {
    av_stream: *mut avformat::AVStream,
}

impl AVStream {
    pub fn new(av_stream: *mut avformat::AVStream) -> AVStream {
        assert!(av_stream.is_not_null());
        debug!("av_stream::new() index = {}", unsafe {(*av_stream).index} as int);
        AVStream {
            av_stream: av_stream,
        }
    }
    pub fn get_type(&self) -> avutil::Enum_AVMediaType {
        unsafe {
            let codec = (*self.av_stream).codec;
            if codec.is_not_null() {
                (*codec).codec_type
            } else {
                avutil::AVMEDIA_TYPE_UNKNOWN
            }
        }
    }
    pub fn get_index(&self) -> int {
        unsafe {
            (*self.av_stream).index as int
        }
    }
}

impl Clone for AVStream {
    fn clone(&self) -> AVStream {
        AVStream {
            av_stream: self.av_stream,
        }
    }
}

pub struct AVStreamIterator {
    nb_streams: c_uint,
    offset: c_uint,
    streams: *mut *mut avformat::AVStream
}

impl Iterator<AVStream> for AVStreamIterator {
    fn next(&mut self) -> Option<AVStream> {
        if self.nb_streams <= self.offset {
            None
        } else {
            unsafe {
                let av_stream = self.streams.offset(self.offset as int);
                self.offset += 1;
                Some(AVStream::new(*av_stream))
            }
        }
    }
}
