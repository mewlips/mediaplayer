use module::Module;
use component::{Component,ComponentType,Extractor,ExtractorComponent};
use ll_avformat;

mod avformat;

pub struct FFmpegModule {
    pub name: &'static str,
}

impl Module for FFmpegModule {
    fn new() -> FFmpegModule {
        FFmpegModule {
            name: "FFmpeg"
        }
    }
    fn get_name(&self) -> &'static str {
        self.name
    }
    fn init(&self) -> bool {
        avformat::av_register_all();
        true
    }
    fn get_component(&self, component_type: ComponentType)
            -> Option<Box<Component>> {
        match component_type {
            ExtractorComponent => {
                let extractor = box FFmpegExtractor::new();
                Some(extractor as Box<Component>)
            }
            _ => None
        }
    }
}

impl Drop for FFmpegModule {
    fn drop(&mut self) {
    }
}

struct FFmpegExtractor {
    context: *mut ll_avformat::AVFormatContext,
}

impl FFmpegExtractor {
    fn new() -> FFmpegExtractor {
        FFmpegExtractor {
            context: avformat::alloc_context()
        }
    }
}

impl Component for FFmpegExtractor {
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

impl Extractor for FFmpegExtractor {
    fn seek(&mut self) -> bool {
        true
    }
}
