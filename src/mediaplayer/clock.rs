use avutil::av_gettime;
use util;
use component::{Component,ComponentStruct,
                AudioDecoderComponent,ExtractorComponent,
                VideoDecoderComponent,ClockComponent,UiComponent};
use message::{Message,MsgStop,MsgPts,MsgExtract,MsgPause,MsgSeek};

pub struct Clock {
    pub component: Option<ComponentStruct>,
    pub media_clock: f64,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            component: Some(ComponentStruct::new(ClockComponent)),
            media_clock: 0f64
        }
    }
    fn get_time() -> f64 {
        unsafe {
            av_gettime() as f64 / 1000_000f64
        }
    }
    pub fn start(&mut self) {
        let component = self.component.take().unwrap();
        spawn(proc() {
            component.wait_for_start();
            let latency = 0.2f64;
            let mut clock = latency;
            let mut paused = false;
            let mut extract_count = 0;
            loop {
                let last_clock = Clock::get_time();
                match component.recv() {
                    Message { from, msg: MsgPts(pts), .. } => {
                        //debug!("Clock: pts {} from {}", pts, from);
                        if from == VideoDecoderComponent ||
                           from == AudioDecoderComponent {
                            if !paused {
                                if clock < pts {
                                    util::usleep(((pts - clock) * 1000_000f64) as int);
                                }
                                component.send(ExtractorComponent, MsgExtract);
                            } else {
                                extract_count += 1;
                            }
                        }
                        component.send(UiComponent, MsgPts(clock));
                        let elapse_time = Clock::get_time() - last_clock;
                        clock += elapse_time; // + 0.0001f64;
                    }
                    Message { msg: MsgStop, .. } => {
                        component.flush();
                        break;
                    }
                    Message { msg: MsgPause, .. } => {
                        if paused {
                            while extract_count > 0 {
                                component.send(ExtractorComponent, MsgExtract);
                                extract_count -= 1;
                            }
                        }
                        paused = !paused;
                    }
                    Message { msg: MsgSeek(seek_pts,_), .. } => {
                        clock = seek_pts;
                    }
                    _ => {
                        error!("unexpected message received");
                        break;
                    }
                }
            }
            info!("stop Clock");
        })
    }
}

impl Drop for Clock {
    fn drop(&mut self) {
        debug!("Clock::drop()");
    }
}

impl Component for Clock {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct {
        self.component.get_mut_ref()
    }
}
