use extra::url;
use extractor::Extractor;
use avcodec;
use avutil;
use video_decoder::VideoDecoder;
use audio_decoder::AudioDecoder;
use video_renderer::VideoRenderer;
use audio_renderer::AudioRenderer;
use std::comm::SharedPort;
use video_scheduler::{VideoBuffer,VideoScheduler};

enum DataSource {
    UrlSource(url::Url),
    FileSource(Path)
}

#[deriving(Eq)]
pub enum Command {
    Start,
}

struct MediaPlayer {
    source: Option<DataSource>,
    extractor: Option<Extractor>,
    video_decoder: Option<VideoDecoder>,
    audio_decoder: Option<AudioDecoder>,
    video_scheduler: Option<VideoScheduler>,
    video_renderer: Option<VideoRenderer>,
    audio_renderer: Option<AudioRenderer>,
    ctrl_chan: Option<Chan<Command>>,
}

impl MediaPlayer {
    pub fn new() -> MediaPlayer {
        MediaPlayer {
            source: None,
            extractor: None,
            video_decoder: None,
            audio_decoder: None,
            video_scheduler: None,
            video_renderer: None,
            audio_renderer: None,
            ctrl_chan: None,
        }
    }
    pub fn set_url_source(&mut self, url: url::Url) {
        self.source = Some(UrlSource(url));
    }
    pub fn set_file_source(&mut self, path: Path) {
        self.source = Some(FileSource(path));
    }
    pub fn prepare(&mut self) -> bool {
        let (ctrl_port, ctrl_chan): (Port<Command>, Chan<Command>) = stream();
        self.ctrl_chan = Some(ctrl_chan);
        let ctrl_port = SharedPort::new(ctrl_port);

        match self.source {
            Some(UrlSource(ref url)) => {
                warn!("Playing url isn't implemented yet! ({})", url.to_str());
                return false;
            }
            Some(FileSource(ref path)) => {
                debug!("prepare: {}", path.display());
                self.extractor = Extractor::new(ctrl_port.clone(), path);
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
        match extractor.get_stream(avutil::AVMEDIA_TYPE_VIDEO, 0) {
            Some(video_stream) => {
                self.video_decoder = VideoDecoder::new(video_stream);
                let video_decoder = self.video_decoder.get_ref();
                let width = video_decoder.width;
                let height = video_decoder.height;
                let pix_fmt = video_decoder.pix_fmt;
                self.video_scheduler = Some(VideoScheduler::new(5));
                self.video_renderer = Some(VideoRenderer::new(width, height, pix_fmt));
            }
            None => {
                debug!("no video stream found");
            }
        }
        match extractor.get_stream(avutil::AVMEDIA_TYPE_AUDIO, 0) {
            Some(audio_stream) => {
                self.audio_decoder = AudioDecoder::new(audio_stream);
                let codec_ctx = self.audio_decoder.get_ref().decoder.codec_ctx.clone();
                self.audio_renderer = AudioRenderer::new(codec_ctx);
            }
            None => {
                debug!("no audio stream found");
            }
        }

        true
    }
    pub fn start(&mut self) {
        // Extrator <--> Video Decoder
        let (vd_port, vd_chan) = stream::<Option<*mut avcodec::AVPacket>>();
        // Extractor <--> Audio Decoder
        let (ad_port, ad_chan) = stream::<Option<*mut avcodec::AVPacket>>();
        // Video Decoder <--> Video Scheduler
        let (vs_port, vs_chan) = stream::<Option<*mut avcodec::AVFrame>>();
        // Video Scheduler <--> Video Renderer
        let (vr_port, vr_chan) = stream::<Option<~VideoBuffer>>();
        // Audio Decoder <--> Audio Renderer
        let (ar_port, ar_chan) = stream::<Option<~[u8]>>();

        self.extractor.get_ref().start(vd_chan, ad_chan);
        self.video_decoder.get_ref().start(vd_port, vs_chan);
        self.audio_decoder.get_ref().start(ad_port, ar_chan);
        self.video_scheduler.get_ref().start(vs_port, vr_chan);
        self.video_renderer.get_ref().start(vr_port);
        self.audio_renderer.get_ref().start(ar_port);

        self.send_cmd(Start);
    }
    pub fn send_cmd(&self, cmd: Command) {
        self.ctrl_chan.get_ref().send(cmd);
    }
}
