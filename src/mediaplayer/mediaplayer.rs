use extra::url;
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
    UrlSource(url::Url),
    FileSource(Path)
}

#[deriving(Eq)]
pub enum Command {
    Start,
}

pub struct MediaPlayer {
    component_mgr: ComponentManager,
    mp_port: Port<bool>,
    source: Option<DataSource>,
    extractor: Option<Extractor>,
    video_decoder: Option<VideoDecoder>,
    audio_decoder: Option<AudioDecoder>,
    clock: Option<Clock>,
    video_renderer: Option<VideoRenderer>,
    audio_renderer: Option<AudioRenderer>,
    ui: Option<UI>,
}

impl MediaPlayer {
    pub fn new() -> MediaPlayer {
        let (mp_port, mp_chan) = Chan::<bool>::new();
        MediaPlayer {
            component_mgr: ComponentManager::new(mp_chan),
            mp_port: mp_port,
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
    pub fn set_url_source(&mut self, url: url::Url) {
        self.source = Some(UrlSource(url));
    }
    pub fn set_file_source(&mut self, path: Path) {
        self.source = Some(FileSource(path));
    }
    pub fn prepare(&mut self) -> bool {
        match self.source {
            Some(UrlSource(ref url)) => {
                warn!("Playing url isn't implemented yet! ({})", url.to_str());
                return false;
            }
            Some(FileSource(ref path)) => {
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
        let extractor = self.extractor.get_mut_ref();
        self.component_mgr.add(extractor);
        match extractor.get_stream(avutil::AVMEDIA_TYPE_VIDEO, 0) {
            Some(video_stream) => {
                self.video_decoder = VideoDecoder::new(video_stream);
                let video_decoder = self.video_decoder.get_mut_ref();
                self.component_mgr.add(video_decoder);
                let width = video_decoder.width;
                let height = video_decoder.height;
                let pix_fmt = video_decoder.pix_fmt;
                self.video_renderer = Some(VideoRenderer::new(width, height, pix_fmt));
                let video_renderer = self.video_renderer.get_mut_ref();
                self.component_mgr.add(video_renderer);
            }
            None => {
                debug!("no video stream found");
            }
        }
        match extractor.get_stream(avutil::AVMEDIA_TYPE_AUDIO, 0) {
            Some(audio_stream) => {
                self.audio_decoder = AudioDecoder::new(audio_stream);
                let audio_decoder = self.audio_decoder.get_mut_ref();
                self.component_mgr.add(audio_decoder);
                let codec_ctx = audio_decoder.decoder.codec_ctx.clone();
                self.audio_renderer = AudioRenderer::new(codec_ctx);
                let audio_renderer = self.audio_renderer.get_mut_ref();
                self.component_mgr.add(audio_renderer);
            }
            None => {
                debug!("no audio stream found");
            }
        }

        self.clock = Some(Clock::new());
        let clock = self.clock.get_mut_ref();
        self.component_mgr.add(clock);

        self.ui = Some(UI::new());
        let ui = self.ui.get_mut_ref();
        self.component_mgr.add(ui);

        true
    }
    pub fn start(&mut self) {
        self.extractor.get_mut_ref().start();
        if self.audio_decoder.is_some() {
            self.audio_decoder.get_mut_ref().start();
            self.audio_renderer.get_mut_ref().start();
        }
        if self.video_decoder.is_some() {
            self.video_decoder.get_mut_ref().start();
            self.video_renderer.get_mut_ref().start();
        }
        self.clock.get_mut_ref().start();
        self.ui.get_mut_ref().start();

        self.component_mgr.start();
    }
    pub fn wait(&self) {
        match self.mp_port.recv() {
            true => {
                info!("mediaplayer stopped");
            }
            false => {
            }
        }
    }
}
