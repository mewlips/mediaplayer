pub trait Component {
    fn get_name(&self) -> &'static str;
    fn prepare(&mut self) -> bool {
        true
    }
    fn start(&mut self) -> bool {
        true
    }
    fn pause(&mut self) -> bool {
        true
    }
    fn stop(&mut self) -> bool {
        true
    }
}

pub trait Extractor : Component {
    fn set_source(&mut self, path: &Path) -> bool;
    fn seek(&mut self) -> bool {
        true
    }
}
