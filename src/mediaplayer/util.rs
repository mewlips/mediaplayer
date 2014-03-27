use std::libc::{c_int,size_t};
use avutil;

pub fn av_strerror(e: i32) -> ~str {
    let mut buf = Vec::from_elem(128, 0_u8);
    let len = buf.len();
    let ptr = buf.as_mut_slice().as_mut_ptr();
    unsafe {
        avutil::av_strerror(-(e as c_int), ptr as *mut i8, len as size_t);
    }
    buf.to_str()
}

mod ffi {
    use std::libc::c_int;
    extern "C" {
        pub fn usleep(usec: c_int) -> c_int;
    }
}

pub fn usleep(usec: int) -> int {
    unsafe {
        ffi::usleep(usec as c_int) as int
    }
}

