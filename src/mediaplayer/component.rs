use std::comm::{TryRecvError,Empty,Disconnected};
use std::fmt;
use message::{Message,MessageData,MsgStart};

#[deriving(PartialEq)]
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
            ManagerComponent       => write!(f, "ComponentManager"),
            ExtractorComponent     => write!(f, "Extractor"),
            AudioDecoderComponent  => write!(f, "AudioDecoder"),
            VideoDecoderComponent  => write!(f, "VideoDecoder"),
            ClockComponent         => write!(f, "Clock"),
            AudioRendererComponent => write!(f, "AudioRenderer"),
            VideoRendererComponent => write!(f, "VideoRenderer"),
            UiComponent            => write!(f, "UI"),
        }
    }
}

pub struct ComponentStruct {
    pub component_type: ComponentType,
    pub mgr_sender: Option<Sender<Message>>,
    pub receiver: Receiver<Message>,
    pub sender: Option<Sender<Message>>,
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
        match self.mgr_sender.get_ref().send_opt(Message {
                from: self.component_type,
                to: to,
                msg: msg
            }) {
            Ok(_) => true,
            Err(_) => false
        }
    }
    pub fn recv(&self) -> Message {
        self.receiver.recv()
    }
    pub fn try_recv(&self) -> Result<Message, TryRecvError> {
        self.receiver.try_recv()
    }
    pub fn flush(&self) {
        loop {
            match self.receiver.try_recv() {
                Ok(_msg) => {
                    debug!("{} flush", self.component_type);
                }
                Err(Empty) => {
                    break
                }
                Err(Disconnected) => {
                    break;
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

