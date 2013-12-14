use std::vec::{from_elem};
use std::libc::{c_int,size_t};
use avutil;

pub fn av_strerror(e: i32) -> ~str {
    let mut buf = from_elem(128, 0_u8);
    buf.as_mut_buf(|p: *mut u8, len: uint| {
        unsafe { avutil::av_strerror(-(e as c_int), p as *mut i8, len as size_t) };
    });
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

