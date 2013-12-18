struct AudioBuffer {
    data: [u8, ..8192],
    offset: uint,
    length: uint,
    copied: bool,
}

impl AudioBuffer {
    pub fn new() -> AudioBuffer {
        AudioBuffer {
            data: [0, ..8192],
            offset: 0,
            length: 0,
            copied: false,
        }
    }
    pub fn copy(&mut self, data: *mut u8, size: int) {
        let mut idx = 0;
        while idx < size {
            self.data[idx] = unsafe { *(data.offset(idx)) };
            idx += 1;
        }
        println("copy");
    }
}
