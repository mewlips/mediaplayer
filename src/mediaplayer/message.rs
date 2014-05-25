use component::ComponentType;
use std::fmt;
use audio_decoder::AudioData;
use video_decoder::VideoData;
use avcodec;
use libc::c_int;

pub struct Message {
    pub from: ComponentType,
    pub to: ComponentType,
    pub msg: MessageData
}

impl Message {
    pub fn new(from: ComponentType, to: ComponentType, msg: MessageData) -> Message {
        Message {
            from: from,
            to: to,
            msg: msg,
        }
    }
}

#[deriving(Clone)]
pub enum MessageData {
    MsgStart,
    MsgStop,
    MsgPause,
    MsgPts(f64),
    MsgExtract,
    MsgPacketData(*mut avcodec::AVPacket),
    MsgVideoData(VideoData),
    MsgAudioData(AudioData),
    MsgError(&'static str),
    MsgEOF,
    MsgSeek(f64,c_int),
    MsgFlush,
}

impl fmt::Show for MessageData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MsgStart           => write!(f, "MsgStart"),
            MsgStop            => write!(f, "MsgStop"),
            MsgPause           => write!(f, "MsgPause"),
            MsgPts(pts)        => write!(f, "MsgPts({})", pts),
            MsgExtract         => write!(f, "MsgExtract"),
            MsgPacketData(_)   => write!(f, "MsgPacketData(..)"),
            MsgVideoData(_)    => write!(f, "MsgVideoData(..)"),
            MsgAudioData(_)    => write!(f, "MsgAudioData(..)"),
            MsgError(_)        => write!(f, "MsgError(..)"),
            MsgEOF             => write!(f, "MsgEOF"),
            MsgSeek(pts,flags) => write!(f, "MsgSeek({},{})", pts, flags),
            MsgFlush           => write!(f, "MsgFlush"),
        }
    }
}
