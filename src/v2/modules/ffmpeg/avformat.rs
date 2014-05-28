use ll_avformat;

pub fn av_register_all() {
    unsafe {
        ll_avformat::av_register_all();
    }
}

pub fn alloc_context() -> *mut ll_avformat::AVFormatContext {
    unsafe {
        ll_avformat::avformat_alloc_context()
    }
}
