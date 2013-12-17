use extra::url;
use extractor::Extractor;
use avcodec;
use avutil;
use videodecoder::VideoDecoder;
use audiodecoder::AudioDecoder;
use videorenderer::VideoRenderer;
use std::comm::SharedPort;
use extra::arc::RWArc;
use extra::dlist::DList;

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
    video_renderer: Option<VideoRenderer>,
    ctrl_chan: Option<Chan<Command>>,
}

impl MediaPlayer {
    pub fn new() -> MediaPlayer {
        MediaPlayer {
            source: None,
            extractor: None,
            video_decoder: None,
            audio_decoder: None,
            video_renderer: None,
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
                self.video_renderer = Some(VideoRenderer::new(width, height, pix_fmt));
            }
            None => {
            }
        }
        match extractor.get_stream(avutil::AVMEDIA_TYPE_AUDIO, 0) {
            Some(audio_stream) => {
                self.audio_decoder = AudioDecoder::new(audio_stream);
                let audio_decoder = self.audio_decoder.get_ref();
            }
            None => {
                debug!("no audio stream found");
            }
        }

        true
    }
    pub fn start(&mut self) {
        let (vd_port, vd_chan): (Port<Option<*mut avcodec::AVPacket>>,
                                 Chan<Option<*mut avcodec::AVPacket>>) = stream();
        let (vr_port, vr_chan): (Port<Option<*mut avcodec::AVFrame>>,
                                 Chan<Option<*mut avcodec::AVFrame>>) = stream();
        let (ad_port, ad_chan): (Port<Option<*mut avcodec::AVPacket>>,
                                 Chan<Option<*mut avcodec::AVPacket>>) = stream();
        let audio_queue = ~RWArc::new(DList::new());

        self.extractor.get_ref().start(vd_chan, audio_queue.clone());
        self.video_decoder.get_ref().start(vd_port, vr_chan);
        self.audio_decoder.get_ref().start(audio_queue.clone());
        self.video_renderer.get_ref().start(vr_port);

        self.send_cmd(Start);
    }
    pub fn send_cmd(&self, cmd: Command) {
        self.ctrl_chan.get_ref().send(cmd);
    }
}
