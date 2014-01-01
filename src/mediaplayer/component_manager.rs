use std::fmt;

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
    pub fn send(&self, to: ComponentType, msg:MessageData) {
        self.mgr_chan.get_ref().send(Message {
            from: self.component_type,
            to: to,
            msg: msg
        });
    }
    pub fn recv(&self) -> Message {
        self.port.recv()
    }
    pub fn try_recv(&self) -> Option<Message> {
        self.port.try_recv()
    }
}

pub struct Message {
    from: ComponentType,
    to: ComponentType,
    msg: MessageData
}

pub enum MessageData {
    MsgStart,
    MsgPts(f64),
    MsgExtract,
    //MsgPacketData(*mut avcodec::AVPacket)
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
        println!("new component add: {}", component.component_type);
    }
    pub fn start(&mut self) {
        let port = self.msg_port.take().unwrap();
        println!("ComponentManager::start()");
        let components = self.components.take().unwrap();
        do spawn {
            for &(component_type, ref chan) in components.iter() {
                chan.send(Message {
                    from: ManagerComponent,
                    to: component_type,
                    msg: MsgStart,
                });
            }
            loop {
                match port.recv() {
                    Message { from, to, msg } => {
                        for &(component_type, ref chan) in components.iter() {
                            if component_type == to {
                                //println!("{} --> {}", from, to);
                                chan.send(Message {
                                    from: from,
                                    to: to,
                                    msg: msg
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
