use std::ptr::mut_null;
use avformat;

struct Extractor {
    fmt_ctx: *mut avformat::AVFormatContext,
}

impl Extractor {
    pub fn new(path: &Path) -> Option<Extractor> {
        let mut extractor = Extractor {
            fmt_ctx: unsafe { avformat::avformat_alloc_context() },
        };
        if extractor.fmt_ctx == mut_null() {
            None
        } else {
            let mut result = path.with_c_str(|path| {
                unsafe { 
                    avformat::avformat_open_input(&mut extractor.fmt_ctx, path,
                                                  mut_null(), mut_null())
                }
            });
            if result < 0 {
                println("avformat_open_input() failed");
                return None;
            }

            result = unsafe {
                avformat::avformat_find_stream_info(extractor.fmt_ctx, mut_null())
            };
            if result < 0 {
                println("avformat_find_stream_info() failed!");
                return None;
            }
            Some(extractor)
        }
    }
}
