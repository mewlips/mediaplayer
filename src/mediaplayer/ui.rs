use component::{Component,ComponentStruct,ManagerComponent,
                ClockComponent,UiComponent};
use message::{Message,MsgEOF,MsgStop,MsgPause};
use util;
use sdl;

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
        do spawn {
            component.wait_for_start();
            loop {
                match sdl::event::poll_event() {
                    sdl::event::QuitEvent => {
                        component.send(ManagerComponent, MsgEOF);
                    }
                    sdl::event::MouseButtonEvent(button, state, _, _) => {
                        match button {
                            sdl::event::LeftMouse if state => {
                                component.send(ClockComponent, MsgPause);
                            }
                            _ => {
                            }
                        }
                    }
                    sdl::event::NoEvent => {
                    }
                    _ => {
                    }
                }
                match component.try_recv() {
                    Some(Message { msg: MsgStop, .. }) => {
                        break;
                    }
                    _ => {
                    }
                }
                util::usleep(10_000);
            }
            info!("stop UI");
        }
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
