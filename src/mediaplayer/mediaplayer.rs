use extra::url;
use extractor::Extractor;

enum DataSource {
    UrlSource(url::Url),
    FileSource(Path)
}

struct MediaPlayer {
    source: Option<DataSource>,
    extractor: Option<Extractor>,
}

impl MediaPlayer {
    pub fn new() -> MediaPlayer {
        MediaPlayer {
            source: None,
            extractor: None,
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
                println!("Playing url isn't implemented yet! ({})", url.to_str());
                return false;
            }
            Some(FileSource(ref path)) => {
                println!("prepare: {}", path.display());
                self.extractor = Extractor::new(path);
                if self.extractor.is_none() {
                    return false;
                }
            }
            None => {
                println!("prepare() error: source not found.");
                return false;
            }
        }
        true
    }
}
