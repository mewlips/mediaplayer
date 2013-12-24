use avcodec;
use extra::arc::RWArc;
use extra::dlist::DList;

pub struct VideoPicture {
    frame: *mut avcodec::AVFrame,
    width: i32,
    height: i32,
    //data: ~[u8],
}

pub struct VideoScheduler {
    max_buffer: int,
    video_pictures: RWArc<DList<~VideoPicture>>,
}

impl VideoScheduler {
    pub fn new(max_buffer: int) -> VideoScheduler {
        VideoScheduler {
            max_buffer: max_buffer,
            video_pictures: RWArc::new(DList::new()),
        }
    }
    pub fn start(&self, vs_port: Port<Option<*mut avcodec::AVFrame>>,
                        vr_chan: Chan<Option<~VideoPicture>>) {
        let max_buffer = self.max_buffer.clone();
        let video_pictures = self.video_pictures.clone();
        do spawn {
            while VideoScheduler::add_frame(max_buffer, video_pictures,
                                            &vs_port, &vr_chan) {
                ;
            }
        }
    }
    fn add_frame(max_buffer: int, video_pictures: RWArc<DList<~VideoPicture>>,
                 vs_port: &Port<Option<*mut avcodec::AVFrame>>,
                 vr_chan: &Chan<Option<~VideoPicture>>) -> bool{
        match vs_port.recv() {
            Some(mut frame) => {
                let buffer = ~VideoPicture {
                    frame: frame.clone(),
                    width: unsafe { (*frame).width },
                    height: unsafe { (*frame).height },
                };
                vr_chan.send(Some(buffer));
                true
            }
            None => {
                vr_chan.send(None);
                false
            }
        }
    }
}
