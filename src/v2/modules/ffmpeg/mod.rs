use ll_avutil;
use module::Module;
use component::{Pipe, Message, MsgAddSender};
use stream::{Stream,Video,Audio,Other};
use libc::types::os::arch::c95::c_uint;

mod result;
mod avformat;
mod avutil;
mod avcodec;

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

    fn get_extractor(&self) -> Option<(SyncSender<Message>, proc():Send)> {
        let (sender, receiver) = sync_channel::<Message>(100);
        Some((sender, proc() {
            let mut extractor = FFmpegExtractor::new();

            loop {
                match receiver.recv() {
                    MsgAddSender(sender) => {
                        debug!("MsgAddSender");
                        extractor.pipe.add_sender(sender);
                    }
                    //_ => continue
                }
            }
        }))
    }

}

impl Drop for FFmpegModule {
    fn drop(&mut self) {
    }
}

pub struct FFmpegExtractor {
    pipe: Pipe,
    context: avformat::AVFormatContext,
    offset: int, // for Iterator
}

impl FFmpegExtractor {
    pub fn new() -> FFmpegExtractor {
        FFmpegExtractor {
            pipe: Pipe::new(),
            context: avformat::AVFormatContext::alloc_context(),
            offset: 0
        }
    }

    fn set_data_source(&mut self, path: &Path) -> bool {
        match self.context.open_input(path) {
            Ok(_) => debug!("open_input()"),
            Err(e) => {
                error!("set_data_source(): {}", e);
                return false;
            }
        }
        match self.context.find_stream_info(None) {
            Ok(_) => debug!("find_stream_info()"),
            Err(e) => {
                error!("set_data_source(): {}", e);
                return false;
            }
        }
        self.context.dump_format(0, path, false);

        true
    }

    fn seek(&mut self) -> bool {
        true
    }
    fn pump(&mut self) -> bool {
        match avcodec::AVPacket::new() {
            Some(mut packet) => {
                match self.context.read_frame(&mut packet) {
                    Ok(_) => {
                        let index = packet.get_raw_ref().stream_index;
                        debug!("index = {}", index);
                        true
                    }
                    Err(e) => {
                        error!("pump(): {}", e);
                        false
                    }
                }
            }
            None => {
                error!("packet allocation failed");
                false
            }
        }
    }
}

impl Iterator<Stream> for FFmpegExtractor {
    fn next(&mut self) -> Option<Stream> {
        if self.context.get_raw_ref().nb_streams <= self.offset as c_uint {
            None
        } else {
            let context = self.context.get_raw_ref();
            let stream = unsafe {
                let stream = *context.streams.offset(self.offset);
                let codec = (*stream).codec;
                let media_type =
                    if codec.is_not_null() {
                        match (*codec).codec_type {
                            ll_avutil::AVMEDIA_TYPE_VIDEO => Video,
                            ll_avutil::AVMEDIA_TYPE_AUDIO => Audio,
                            _ => Other
                        }
                    } else {
                        Other
                    };
                let index = (*stream).index as int;
                Stream { media_type: media_type, index: index }
            };
            self.offset += 1;
            Some(stream)
        }
    }
}
