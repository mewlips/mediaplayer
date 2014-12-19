use std::comm::{TryRecvError,Empty,Disconnected};
use std::fmt;
use message::{Message,MessageData};
use message::MessageData::{MsgStart};

#[deriving(PartialEq,Clone)]
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
            ComponentType::ManagerComponent       => write!(f, "ComponentManager"),
            ComponentType::ExtractorComponent     => write!(f, "Extractor"),
            ComponentType::AudioDecoderComponent  => write!(f, "AudioDecoder"),
            ComponentType::VideoDecoderComponent  => write!(f, "VideoDecoder"),
            ComponentType::ClockComponent         => write!(f, "Clock"),
            ComponentType::AudioRendererComponent => write!(f, "AudioRenderer"),
            ComponentType::VideoRendererComponent => write!(f, "VideoRenderer"),
            ComponentType::UiComponent            => write!(f, "UI"),
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
        match self.mgr_sender.as_ref().unwrap().send_opt(Message {
                from: self.component_type.clone(),
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
            Message { from: ComponentType::ManagerComponent, msg: MsgStart, .. } => {
                info!("start {}", self.component_type);
            }
            _ => {
                panic!("unexpected message received");
            }
        }
    }
}

pub trait Component {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct;
}

