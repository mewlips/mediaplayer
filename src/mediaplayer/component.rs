use std::comm::{TryRecvResult,Empty,Disconnected,Data};
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

impl fmt::Show for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
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
    mgr_sender: Option<Sender<Message>>,
    receiver: Receiver<Message>,
    sender: Option<Sender<Message>>,
}

impl ComponentStruct {
    pub fn new(component_type: ComponentType) -> ComponentStruct {
        let (sender, receiver) = channel::<Message>();
        ComponentStruct {
            component_type: component_type,
            mgr_sender: None,
            receiver: receiver,
            sender: Some(sender),
        }
    }
    pub fn set_mgr_sender(&mut self, sender: Sender<Message>) {
        self.mgr_sender= Some(sender);
    }
    pub fn take_sender(&mut self) -> Sender<Message> {
        self.sender.take().unwrap()
    }
    pub fn send(&self, to: ComponentType, msg:MessageData) -> bool {
        self.mgr_sender.get_ref().try_send(Message {
            from: self.component_type,
            to: to,
            msg: msg
        })
    }
    pub fn recv(&self) -> Message {
        self.receiver.recv()
    }
    pub fn try_recv(&self) -> TryRecvResult<Message> {
        self.receiver.try_recv()
    }
    pub fn flush(&self) {
        loop {
            match self.receiver.try_recv() {
                Empty => {
                    break
                }
                Disconnected => {
                    break;
                }
                Data(_msg) => {
                    debug!("{} flush", self.component_type);
                }
            }
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

