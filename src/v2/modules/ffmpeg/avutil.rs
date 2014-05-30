use libc::{c_int,c_uint,size_t};
use libc::funcs::posix88::unistd;
use ll_avutil;

pub type AvError = c_int;

pub fn av_strerror(err: int) -> String {
    let mut buf = Vec::from_elem(128, 0_u8);
    let len = buf.len();
    let ptr = buf.as_mut_slice().as_mut_ptr();
    unsafe {
        ll_avutil::av_strerror(
            -(err as c_int), ptr as *mut i8, len as size_t);
    }
    buf.to_str()
}
