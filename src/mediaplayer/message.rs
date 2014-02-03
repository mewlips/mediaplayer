use component::ComponentType;
use std::fmt;
use audio_decoder::AudioData;
use video_decoder::VideoData;
use avcodec;
use std::libc::c_int;

pub struct Message {
    from: ComponentType,
    to: ComponentType,
    msg: MessageData
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
    MsgVideoData(~VideoData),
    MsgAudioData(~AudioData),
    MsgError(~str),
    MsgEOF,
    MsgSeek(f64,c_int),
    MsgFlush,
}

impl fmt::Show for MessageData {
    fn fmt(t: &MessageData, f: &mut fmt::Formatter) {
        match *t {
            MsgStart           => write!(f.buf, "MsgStart"),
            MsgStop            => write!(f.buf, "MsgStop"),
            MsgPause           => write!(f.buf, "MsgPause"),
            MsgPts(pts)        => write!(f.buf, "MsgPts({})", pts),
            MsgExtract         => write!(f.buf, "MsgExtract"),
            MsgPacketData(_)   => write!(f.buf, "MsgPacketData(..)"),
            MsgVideoData(_)    => write!(f.buf, "MsgVideoData(..)"),
            MsgAudioData(_)    => write!(f.buf, "MsgAudioData(..)"),
            MsgError(_)        => write!(f.buf, "MsgError(..)"),
            MsgEOF             => write!(f.buf, "MsgEOF"),
            MsgSeek(pts,flags) => write!(f.buf, "MsgSeek({},{})", pts, flags),
            MsgFlush           => write!(f.buf, "MsgFlush"),
        }
    }
}
