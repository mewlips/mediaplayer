use std::comm::{Data};
use component::{Component,ComponentStruct,ManagerComponent,
                ClockComponent,UiComponent,ExtractorComponent};
use message::{Message,MsgEOF,MsgStop,MsgPause,MsgPts,MsgSeek};
use util;
use sdl;
use avformat;

pub struct UI {
    component: Option<ComponentStruct>,
}

impl UI {
    pub fn new() -> UI {
        UI {
            component: Some(ComponentStruct::new(UiComponent)),
        }
    }
    pub fn start(&mut self) {
        let component = self.component.take().unwrap();
        spawn(proc() {
            component.wait_for_start();
            let mut clock = 0f64;
            loop {
                match sdl::event::poll_event() {
                    sdl::event::QuitEvent => {
                        component.send(ManagerComponent, MsgEOF);
                    }
                    sdl::event::MouseButtonEvent(sdl::event::LeftMouse, true, _, _) => {
                        component.send(ClockComponent, MsgPause);
                    }
                    sdl::event::KeyEvent(sdl::event::LeftKey, true, _, _) => {
                        let msg = MsgSeek(clock - 10.0f64, avformat::AVSEEK_FLAG_BACKWARD);
                        component.send(ExtractorComponent, msg.clone());
                        component.send(ClockComponent, msg);
                    }
                    sdl::event::KeyEvent(sdl::event::RightKey, true, _, _) => {
                        let msg = MsgSeek(clock + 10.0f64, 0);
                        component.send(ExtractorComponent, msg.clone());
                        component.send(ClockComponent, msg);
                    }
                    sdl::event::NoEvent => {
                    }
                    _ => {
                    }
                }
                match component.try_recv() {
                    Data(Message { msg: MsgStop, .. }) => {
                        break;
                    }
                    Data(Message { msg: MsgPts(pts), ..}) => {
                        print!("\r{}", pts);
                        ::std::io::stdio::flush();
                        clock = pts;
                    }
                    _ => {
                    }
                }
                util::usleep(10_000);
            }
            info!("stop UI");
        })
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        debug!("UI::drop()");
    }
}

impl Component for UI {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct {
        self.component.get_mut_ref()
    }
}
