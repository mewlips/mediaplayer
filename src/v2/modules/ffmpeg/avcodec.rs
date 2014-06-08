use ll_avcodec;
use ll_avutil;
use std::mem::{size_of, transmute};

pub struct AVPacket {
    raw: *mut ll_avcodec::AVPacket
}

impl AVPacket {
    pub fn new() -> Option<AVPacket> {
        let size = size_of::<ll_avcodec::AVPacket>();
        let packet: *mut ll_avcodec::AVPacket = unsafe {
            transmute(ll_avutil::av_malloc(size as u64))
        };
        if packet.is_null() {
            None
        } else {
            unsafe {
                ll_avcodec::av_init_packet(packet);
            }
            Some(AVPacket {
                raw: packet
            })
        }
    }
    pub fn get_raw_ref(&mut self) -> &mut ll_avcodec::AVPacket {
        unsafe {
            &mut (*self.raw)
        }
    }
}
