use libc::{c_int,c_uint,size_t};
use libc::funcs::posix88::unistd;
use avutil;

pub fn av_strerror(e: i32) -> String {
    let mut buf = Vec::from_elem(128, 0_u8);
    let len = buf.len();
    let ptr = buf.as_mut_slice().as_mut_ptr();
    unsafe {
        avutil::av_strerror(-(e as c_int), ptr as *mut i8, len as size_t);
    }
    buf.to_string()
}

pub fn usleep(usec: int) -> int {
    unsafe {
        unistd::usleep(usec as c_uint) as int
    }
}

