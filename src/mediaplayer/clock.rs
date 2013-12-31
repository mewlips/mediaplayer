use avutil::av_gettime;
use component_manager::{Component,ComponentId,Message};

pub struct Clock {
    component_id: Option<ComponentId>,
    chan: Option<SharedChan<Message>>,
    media_clock: f64,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            component_id: None,
            chan: None,
            media_clock: 0f64
        }
    }
}

impl Drop for Clock {
    fn drop(&mut self) {
        debug!("Clock::drop()");
    }
}

impl Component for Clock {
    fn set_id(&mut self, id: ComponentId) {
        self.component_id = Some(id);
    }
    fn get_id(&self) -> Option<ComponentId> {
        self.component_id
    }
    fn get_name(&self) -> &str {
        "Clock"
    }
    fn set_chan(&mut self, chan: SharedChan<Message>) {
        self.chan = Some(chan);
    }
}

