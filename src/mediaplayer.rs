use extractor::Extractor;
use avutil;
use video_decoder::VideoDecoder;
use audio_decoder::AudioDecoder;
use clock::Clock;
use video_renderer::VideoRenderer;
use audio_renderer::AudioRenderer;
use component_manager::{ComponentManager};
use ui::UI;

enum DataSource {
    FileSource(Path)
}

#[deriving(PartialEq)]
pub enum Command {
    Start,
}

pub struct MediaPlayer {
    pub component_mgr: ComponentManager,
    pub mp_receiver: Receiver<bool>,
    pub source: Option<DataSource>,
    pub extractor: Option<Extractor>,
    pub video_decoder: Option<VideoDecoder>,
    pub audio_decoder: Option<AudioDecoder>,
    pub clock: Option<Clock>,
    pub video_renderer: Option<VideoRenderer>,
    pub audio_renderer: Option<AudioRenderer>,
    pub ui: Option<UI>,
}

impl MediaPlayer {
    pub fn new() -> MediaPlayer {
        let (mp_sender, mp_receiver) = channel::<bool>();
        MediaPlayer {
            component_mgr: ComponentManager::new(mp_sender),
            mp_receiver: mp_receiver,
            source: None,
            extractor: None,
            video_decoder: None,
            audio_decoder: None,
            clock: None,
            video_renderer: None,
            audio_renderer: None,
            ui: None,
        }
    }
    pub fn set_file_source(&mut self, path: Path) {
        self.source = Some(DataSource::FileSource(path));
    }
    pub fn prepare(&mut self) -> bool {
        match self.source {
            Some(DataSource::FileSource(ref path)) => {
                debug!("prepare: {}", path.display());
                self.extractor = Extractor::new(path);
                if self.extractor.is_none() {
                    return false;
                }
            }
            None => {
                error!("prepare() error: source not found.");
                return false;
            }
        }
        self.component_mgr.add(self.extractor.as_mut().unwrap());
        match self.extractor.as_mut().unwrap().get_stream(avutil::AVMEDIA_TYPE_VIDEO, 0) {
            Some(video_stream) => {
                self.video_decoder = VideoDecoder::new(video_stream);
                self.component_mgr.add(self.video_decoder.as_mut().unwrap());
                let width = self.video_decoder.as_mut().unwrap().width;
                let height = self.video_decoder.as_mut().unwrap().height;
                let pix_fmt = self.video_decoder.as_mut().unwrap().pix_fmt;
                self.video_renderer = Some(VideoRenderer::new(width, height, pix_fmt));
                self.component_mgr.add(self.video_renderer.as_mut().unwrap());
            }
            None => {
                debug!("no video stream found");
            }
        }
        match self.extractor.as_mut().unwrap().get_stream(avutil::AVMEDIA_TYPE_AUDIO, 0) {
            Some(audio_stream) => {
                self.audio_decoder = AudioDecoder::new(audio_stream);
                self.component_mgr.add(self.audio_decoder.as_mut().unwrap());
                let codec_ctx = self.audio_decoder.as_mut().unwrap().decoder.codec_ctx.clone();
                self.audio_renderer = AudioRenderer::new(codec_ctx);
                self.component_mgr.add(self.audio_renderer.as_mut().unwrap());
            }
            None => {
                debug!("no audio stream found");
            }
        }

        self.clock = Some(Clock::new());
        let clock = self.clock.as_mut().unwrap();
        self.component_mgr.add(clock);

        self.ui = Some(UI::new());
        let ui = self.ui.as_mut().unwrap();
        self.component_mgr.add(ui);

        true
    }
    pub fn start(&mut self) {
        self.extractor.as_mut().unwrap().start();
        if self.audio_decoder.is_some() {
            self.audio_decoder.as_mut().unwrap().start();
            self.audio_renderer.as_mut().unwrap().start();
        }
        if self.video_decoder.is_some() {
            self.video_decoder.as_mut().unwrap().start();
            self.video_renderer.as_mut().unwrap().start();
        }
        self.clock.as_mut().unwrap().start();
        self.ui.as_mut().unwrap().start();

        self.component_mgr.start();
    }
    pub fn wait(&self) {
        match self.mp_receiver.recv() {
            true => {
                info!("mediaplayer stopped");
            }
            false => {
            }
        }
    }
}
