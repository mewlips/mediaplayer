use std::fmt;
use avcodec;
use audio_decoder::AudioData;
use video_decoder::VideoData;

#[deriving(Eq)]
pub enum ComponentType {
    ManagerComponent,
    ExtractorComponent,
    AudioDecoderComponent,
    VideoDecoderComponent,
    ClockComponent,
    AudioRendererComponent,
    VideoRendererComponent,
}

impl fmt::Default for ComponentType {
    fn fmt(t: &ComponentType, f: &mut fmt::Formatter) {
        match *t {
            ManagerComponent       => write!(f.buf, "ComponentManager"),
            ExtractorComponent     => write!(f.buf, "Extractor"),
            AudioDecoderComponent  => write!(f.buf, "AudioDecoder"),
            VideoDecoderComponent  => write!(f.buf, "VideoDecoder"),
            ClockComponent         => write!(f.buf, "Clock"),
            AudioRendererComponent => write!(f.buf, "AudioRenderer"),
            VideoRendererComponent => write!(f.buf, "VideoRenderer"),
        }
    }
}

pub struct ComponentStruct {
    component_type: ComponentType,
    mgr_chan: Option<SharedChan<Message>>,
    port: Port<Message>,
    chan: Option<Chan<Message>>,
}

impl ComponentStruct {
    pub fn new(component_type: ComponentType) -> ComponentStruct {
        let (port, chan) = Chan::<Message>::new();
        ComponentStruct {
            component_type: component_type,
            mgr_chan: None,
            port: port,
            chan: Some(chan),
        }
    }
    pub fn set_mgr_chan(&mut self, chan: SharedChan<Message>) {
        self.mgr_chan = Some(chan);
    }
    pub fn take_chan(&mut self) -> Chan<Message> {
        self.chan.take().unwrap()
    }
    pub fn send(&self, to: ComponentType, msg:MessageData) -> bool {
        self.mgr_chan.get_ref().try_send(Message {
            from: self.component_type,
            to: to,
            msg: msg
        })
    }
    pub fn recv(&self) -> Message {
        self.port.recv()
    }
    pub fn flush(&self) {
        while self.port.try_recv().is_some() {
            debug!("{} flush", self.component_type);
        }
    }
}

pub struct Message {
    from: ComponentType,
    to: ComponentType,
    msg: MessageData
}

#[deriving(Clone)]
pub enum MessageData {
    MsgPing,
    MsgStart,
    MsgStop,
    MsgPts(f64),
    MsgExtract,
    MsgPacketData(*mut avcodec::AVPacket),
    MsgVideoData(~VideoData),
    MsgAudioData(~AudioData),
    MsgError(~str),
    MsgEOF,
}

impl fmt::Default for MessageData {
    fn fmt(t: &MessageData, f: &mut fmt::Formatter) {
        match *t {
            MsgPing          => write!(f.buf, "MsgPing"),
            MsgStart         => write!(f.buf, "MsgStart"),
            MsgStop          => write!(f.buf, "MsgStop"),
            MsgPts(pts)      => write!(f.buf, "MsgPts({})", pts),
            MsgExtract       => write!(f.buf, "MsgExtract"),
            MsgPacketData(_) => write!(f.buf, "MsgPacketData(..)"),
            MsgVideoData(_)  => write!(f.buf, "MsgVideoData(..)"),
            MsgAudioData(_)  => write!(f.buf, "MsgAudioData(..)"),
            MsgError(_)      => write!(f.buf, "MsgError(..)"),
            MsgEOF           => write!(f.buf, "MsgEOF"),
        }
    }
}

pub trait Component {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct;
}

pub struct ComponentManager {
    priv components: Option<~[(ComponentType, Chan<Message>)]>,
    priv msg_port: Option<Port<Message>>,
    priv msg_chan: SharedChan<Message>,
}

impl ComponentManager {
    pub fn new() -> ComponentManager {
        let (port, chan) = SharedChan::<Message>::new();
        ComponentManager {
            components: Some(~[]),
            msg_port: Some(port),
            msg_chan: chan,
        }
    }
    pub fn add(&mut self, component: &mut Component) {
        let component = component.get();
        let component_type = component.component_type;
        let chan = component.take_chan();
        component.set_mgr_chan(self.msg_chan.clone());
        self.components.get_mut_ref().push((component_type, chan));
        info!("new component add: {}", component.component_type);
    }
    pub fn start(&mut self) {
        let port = self.msg_port.take().unwrap();
        debug!("ComponentManager::start()");
        let components = self.components.take().unwrap();
        do spawn {
            let broadcast = |msg: MessageData| {
                for &(component_type, ref chan) in components.iter() {
                    chan.send(Message {
                        from: ManagerComponent,
                        to: component_type,
                        msg: msg.clone() });
                }
            };
            broadcast(MsgStart);
            loop {
                match port.recv() {
                    Message { from, to, msg } => {
                        //debug!("from = {}, to = {}, msg = {}", from, to, msg);
                        if to == ManagerComponent {
                            match msg {
                                MsgPing => {
                                    // Nothing to do
                                }
                                MsgError(ref _err) => {
                                    // TODO
                                }
                                MsgEOF => {
                                    debug!("MsgEOF received");
                                    broadcast(MsgStop);
                                    break;
                                }
                                _ => {
                                }
                            }
                        } else {
                            for &(component_type, ref chan) in components.iter() {
                                if component_type == to {
                                    //println!("{} --> {}", from, to);
                                    chan.send(Message {
                                        from: from,
                                        to: to,
                                        msg: msg
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            debug!("stop ComponentManager");
        }
    }
    pub fn stop(&self) {
        self.msg_chan.send(Message {
            from: ManagerComponent,
            to: ManagerComponent,
            msg: MsgEOF
        });
    }
    pub fn ping(&self) -> bool {
        self.msg_chan.try_send(Message {
            from: ManagerComponent,
            to: ManagerComponent,
            msg: MsgPing
        })
    }
}
