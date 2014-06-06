use libc::types::os::arch::c95::{c_int,size_t};
use std::fmt;
use std::str::SendStr;
use ll_avutil;

pub type AvResult<T> = Result<T, AvError>;

pub struct AvError {
    error: c_int,
    message: SendStr
}

impl AvError {
    pub fn new<T: IntoMaybeOwned<'static>>(msg: T, error: c_int)
            -> AvError {
        AvError {
            error: error,
            message: msg.into_maybe_owned()
        }
    }
}

impl fmt::Show for AvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = Vec::from_elem(128, 0_u8);
        let len = buf.len();
        let ptr = buf.as_mut_slice().as_mut_ptr();
        unsafe {
            ll_avutil::av_strerror(self.error, ptr as *mut i8, len as size_t);
        }
        write!(f, "{} ({})", self.message, buf.into_ascii().into_str())
    }
}
