use extra::url;
use extractor::Extractor;
use avcodec;
use avutil;
use videodecoder::VideoDecoder;
use videorenderer::VideoRenderer;

enum DataSource {
    UrlSource(url::Url),
    FileSource(Path)
}

struct MediaPlayer {
    source: Option<DataSource>,
    extractor: Option<Extractor>,
    video_decoder: Option<VideoDecoder>,
    video_renderer: Option<VideoRenderer>,
}

impl MediaPlayer {
    pub fn new() -> MediaPlayer {
        MediaPlayer {
            source: None,
            extractor: None,
            video_decoder: None,
            video_renderer: None,
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
        match extractor.get_stream(avutil::AVMEDIA_TYPE_VIDEO, 0) {
            Some(video_stream) => {
                self.video_decoder = VideoDecoder::new(video_stream);
                let video_decoder = self.video_decoder.get_ref();
                let width = video_decoder.get_width();
                let height = video_decoder.get_height();
                let pix_fmt = video_decoder.get_pix_fmt();
                self.video_renderer = Some(VideoRenderer::new(width, height, pix_fmt));
            }
            None => {
            }
        }

        // TODO
        /*
        match extractor.get_stream(avutil::AVMEDIA_TYPE_AUDIO, 0) {
            Some(audio_stream) => {
            }
            None => {
            }
        }
        */

        true
    }
    pub fn start(&mut self) {
        let (vd_port, vd_chan): (Port<*mut avcodec::AVPacket>,
                                 Chan<*mut avcodec::AVPacket>) = stream();
        let (vr_port, vr_chan): (Port<*mut avcodec::AVFrame>,
                                 Chan<*mut avcodec::AVFrame>) = stream();

        self.extractor.get_ref().start(vd_chan);
        self.video_decoder.get_ref().start(vd_port, vr_chan);
        self.video_renderer.get_ref().start(vr_port);
    }
}
