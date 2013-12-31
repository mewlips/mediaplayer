use avutil::av_gettime;
use component_manager::Component;

pub struct Clock {
    component_id: int,
    media_clock: f64,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            component_id: -1,
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
    fn set_id(&mut self, id: int) {
        self.component_id = id;
    }
    fn get_id(&self) -> int {
        self.component_id
    }
    fn get_name(&self) -> &str {
        "Clock"
    }
}

