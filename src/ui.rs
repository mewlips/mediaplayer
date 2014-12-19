use component::{Component,ComponentStruct};
use component::ComponentType::{ManagerComponent,ClockComponent,
                               UiComponent,ExtractorComponent};
use message::{Message,MessageData};
use message::MessageData::{MsgEOF,MsgStop,MsgPause,MsgPts,MsgSeek};
use util;
use sdl;
use avformat;

pub struct UI {
    pub component: Option<ComponentStruct>,
}

impl UI {
    pub fn new() -> UI {
        UI {
            component: Some(ComponentStruct::new(UiComponent)),
        }
    }
    pub fn start(&mut self) {
        let component = self.component.take().unwrap();
        spawn(move || {
            component.wait_for_start();
            let mut clock = 0f64;
            loop {
                match sdl::event::poll_event() {
                    sdl::event::Event::Quit => {
                        component.send(ManagerComponent, MsgEOF);
                    }
                    sdl::event::Event::MouseButton(sdl::event::Mouse::Left, true, _, _) => {
                        component.send(ClockComponent, MsgPause);
                    }
                    sdl::event::Event::Key(sdl::event::Key::Left, true, _, _) => {
                        let msg = MsgSeek(clock - 10.0f64, avformat::AVSEEK_FLAG_BACKWARD);
                        component.send(ExtractorComponent, msg.clone());
                        component.send(ClockComponent, msg);
                    }
                    sdl::event::Event::Key(sdl::event::Key::Right, true, _, _) => {
                        let msg = MsgSeek(clock + 10.0f64, 0);
                        component.send(ExtractorComponent, msg.clone());
                        component.send(ClockComponent, msg);
                    }
                    sdl::event::Event::None => {
                    }
                    _ => {
                    }
                }
                match component.try_recv() {
                    Ok(Message { msg: MsgStop, .. }) => {
                        break;
                    }
                    Ok(Message { msg: MsgPts(pts), ..}) => {
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
        self.component.as_mut().unwrap()
    }
}
