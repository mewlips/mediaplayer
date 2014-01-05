use std::fmt;
use message::{Message,MessageData,MsgStart};

#[deriving(Eq)]
pub enum ComponentType {
    ManagerComponent,
    ExtractorComponent,
    AudioDecoderComponent,
    VideoDecoderComponent,
    ClockComponent,
    AudioRendererComponent,
    VideoRendererComponent,
    UiComponent,
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
            UiComponent            => write!(f.buf, "UI"),
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
    pub fn try_recv(&self) -> Option<Message> {
        self.port.try_recv()
    }
    pub fn flush(&self) {
        while self.port.try_recv().is_some() {
            debug!("{} flush", self.component_type);
        }
    }
    pub fn wait_for_start(&self) {
        match self.recv() {
            Message { from: ManagerComponent, msg: MsgStart, .. } => {
                info!("start {}", self.component_type);
            }
            _ => {
                fail!("unexpected message received");
            }
        }
    }
}

pub trait Component {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct;
}

