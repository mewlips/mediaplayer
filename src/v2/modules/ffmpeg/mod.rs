use module::Module;
use component::{Component,Extractor};
use modules::ffmpeg::avformat::{AVFormatContext};

mod avformat;
mod avutil;

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
        debug!("{} initialized.", self.get_name());
        true
    }
    fn get_extractor(&self) -> Option<Box<Extractor>> {
        let extractor: Box<Extractor> = box FFmpegExtractor::new();
        Some(extractor)
    }
}

impl Drop for FFmpegModule {
    fn drop(&mut self) {
    }
}

struct FFmpegExtractor {
    context: AVFormatContext,
    path: Option<Path>,
}

impl FFmpegExtractor {
    fn new() -> FFmpegExtractor {
        match AVFormatContext::alloc_context() {
            Some(ctx) => {
                FFmpegExtractor {
                    context: ctx,
                    path: None
                }
            }
            None => {
                fail!("alloc_context() failed");
            }
        }
    }
}

impl Component for FFmpegExtractor {
    fn get_name(&self) -> &'static str {
        "FFmpegExtractor"
    }
    fn prepare(&mut self) -> bool {
        match self.path {
            Some(ref path) => {
                self.context.open_input(path);
                true
            }
            None => {
                error!("no path!");
                false
            }
        }
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
    fn set_source(&mut self, path: &Path) -> bool {
        match self.path {
            Some(_) => false,
            None => {
                self.path = Some(path.clone());
                true
            }
        }
    }
    fn seek(&mut self) -> bool {
        true
    }
}

impl Drop for FFmpegExtractor {
    fn drop(&mut self) {
        debug!("drop FFmpegExtractor");
        self.context.free_context();
    }
}
