use ll_avformat;
use std::ptr::mut_null;

pub fn av_register_all() {
    unsafe {
        ll_avformat::av_register_all();
    }
}

pub struct AVFormatContext {
    context: *mut ll_avformat::AVFormatContext,
}

impl AVFormatContext {
    pub fn alloc_context() -> Option<AVFormatContext> {
        let ctx = unsafe { ll_avformat::avformat_alloc_context() };
        if ctx.is_null() {
            None
        } else {
            Some(AVFormatContext {
                context: ctx,
            })
        }
    }

    pub fn free_context(&mut self) {
        unsafe {
            ll_avformat::avformat_free_context(self.context);
            self.context = mut_null();
        }
    }

    pub fn open_input(&mut self, path: &Path) {
        let mut result = path.with_c_str(|path| {
            unsafe {
                ll_avformat::avformat_open_input(
                    &mut self.context,
                    path,
                    mut_null(),
                    mut_null()) // TODO: pass dictionary
            }
        });

    }
}
