use extra::url;
use extractor::Extractor;
use avcodec;
use avutil;
use video_decoder::{VideoData,VideoDecoder};
use audio_decoder::{AudioData,AudioDecoder};
use video_renderer::VideoRenderer;
use audio_renderer::AudioRenderer;
use component_manager::{Component,ComponentManager};
use util;

enum DataSource {
    UrlSource(url::Url),
    FileSource(Path)
}

#[deriving(Eq)]
pub enum Command {
    Start,
}

pub struct MediaPlayer<'a> {
    component_mgr: ComponentManager<'a>,
    source: Option<DataSource>,
    extractor: Option<Extractor>,
    video_decoder: Option<VideoDecoder>,
    audio_decoder: Option<AudioDecoder>,
    video_renderer: Option<VideoRenderer>,
    audio_renderer: Option<AudioRenderer>,
}

impl<'a> MediaPlayer<'a> {
    pub fn new() -> MediaPlayer<'a> {
        MediaPlayer {
            component_mgr: ComponentManager::<'a>::new(),
            source: None,
            extractor: None,
            video_decoder: None,
            audio_decoder: None,
            video_renderer: None,
            audio_renderer: None,
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
                self.component_mgr.add(self.video_renderer.get_mut_ref());
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
                self.component_mgr.add(self.audio_renderer.get_mut_ref());
            }
            None => {
                debug!("no audio stream found");
            }
        }

        true
    }
    pub fn start(&mut self) {
        // Extrator --> Video Decoder
        let (vd_port, vd_chan) = Chan::<Option<*mut avcodec::AVPacket>>::new();
        // Extractor --> Audio Decoder
        let (ad_port, ad_chan) = Chan::<Option<*mut avcodec::AVPacket>>::new();
        // Video Scheduler --> Video Renderer
        let (vr_port, vr_chan) = Chan::<Option<~VideoData>>::new();
        // Audio Decoder --> Audio Renderer
        let (ar_port, ar_chan) = Chan::<Option<~AudioData>>::new();

        self.component_mgr.start();
        self.video_decoder.get_ref().start(vd_port, vr_chan);
        self.audio_decoder.get_mut_ref().start(ad_port, ar_chan);
        self.extractor.get_ref().start(vd_chan, ad_chan);
        self.video_renderer.get_ref().start(vr_port);
        self.audio_renderer.get_ref().start(ar_port);

    }
}
