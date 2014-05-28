pub enum ComponentType {
    ExtractorComponent,
    AudioDecoderComponent,
    VideoDecoderComponent,
    AudioRendererComponent,
    VideoRendererComponent,
}

pub trait Component {
    fn prepare(&mut self) -> bool;
    fn start(&mut self) -> bool;
    fn pause(&mut self) -> bool;
    fn stop(&mut self) -> bool;
}

pub trait Extractor {
    fn seek(&mut self) -> bool;
}
