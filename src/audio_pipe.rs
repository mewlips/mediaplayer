use libc::c_int;
use libc;
use std::mem::transmute;

pub struct AudioPipe {
    pub pipe_input: c_int,
}

impl AudioPipe {
    pub fn new(pipe_input: c_int) -> AudioPipe {
        AudioPipe {
            pipe_input: pipe_input,
        }
    }
    pub fn copy_to(&mut self, data: *mut u8, size: uint) {
        unsafe {
            let result = libc::funcs::posix88::unistd::read(
                            self.pipe_input, transmute(data), size as u64);
            if result < 0 {
                error!("read result = {}", result);
            }
        }
    }
}
