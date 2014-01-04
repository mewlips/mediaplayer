use avutil::av_gettime;
use util;
use component_manager::{Component,ComponentStruct,AudioDecoderComponent,
                        ManagerComponent,ExtractorComponent,
                        VideoDecoderComponent,ClockComponent,
                        Message,MsgStart,MsgStop,MsgPts,MsgExtract};

pub struct Clock {
    component: Option<ComponentStruct>,
    media_clock: f64,
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
        do spawn {
            match component.recv() {
                Message { from: ManagerComponent, msg: MsgStart, .. } => {
                    info!("start Clock");
                }
                _ => {
                    fail!("unexpected message received");
                }
            }

            let mut clock = 0f64;
            loop {
                let last_clock = Clock::get_time();
                match component.recv() {
                    Message { from, msg: MsgPts(pts), .. } => {
                        //debug!("Clock: pts {} from {}", pts, from);
                        if from == VideoDecoderComponent ||
                           from == AudioDecoderComponent {
                            if clock < pts {
                                util::usleep(((pts - clock) * 1000_000f64) as int);
                            }
                            component.send(ExtractorComponent, MsgExtract);
                        }
                    }
                    Message { msg: MsgStop, .. } => {
                        break;
                    }
                    _ => {
                        error!("unexpected message received");
                        break;
                    }
                }
                let elapse_time = Clock::get_time() - last_clock;
                clock += elapse_time; // + 0.0001f64;
                //debug!("current = {}", clock);
            }
            info!("stop Clock");
        }
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
