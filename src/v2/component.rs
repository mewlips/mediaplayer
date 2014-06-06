use stream::{Stream};

pub trait Component {
    fn prepare(&mut self) -> bool;
    fn start(&mut self)   -> bool;
    fn pause(&mut self)   -> bool;
    fn stop(&mut self)    -> bool;

    fn set_source(&mut self) -> bool { true }
    fn set_sink(&mut self)   -> bool { true }
}

pub trait Extractor : Iterator<Stream> {
    fn set_data_source(&mut self, path: &Path) -> bool;
    fn seek(&mut self) -> bool;
    fn pump(&mut self) -> bool;
}

pub trait Clock {
}

pub trait Decoder {
    fn decode(&mut self) -> bool;
}

pub trait VideoDecoder : Decoder {
}

pub trait AudioDecoder : Decoder {
}

pub trait Renderer {
    fn render(&mut self) -> bool;
}

pub trait VideoRenderer : Renderer {
}

pub trait AudioRenderer : Renderer {
}
