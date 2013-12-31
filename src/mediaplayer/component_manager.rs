pub trait Component {
    fn set_id(&mut self, id: ComponentId);
    fn get_id(&self) -> Option<ComponentId>;
    fn get_name(&self) -> &str;
    fn set_chan(&mut self, chan: SharedChan<Message>);
    //fn take_chan(&mut self) -> Option<SharedChan<Message>>;
}

pub type ComponentId = int;

pub struct Message(ComponentId, MessageData);
pub enum MessageData {
    MsgPts(f64),
}

pub struct ComponentManager<'a> {
    priv components: ~[&'a mut Component],
    priv last_id: ComponentId,
    priv msg_port: Option<Port<Message>>,
    priv msg_chan: SharedChan<Message>,
}

impl<'a> ComponentManager<'a> {
    pub fn new() -> ComponentManager<'a> {
        let (port, chan) = SharedChan::<Message>::new();
        ComponentManager {
            components: ~[],
            last_id: -1,
            msg_port: Some(port),
            msg_chan: chan,
        }
    }
    pub fn add(&mut self, component: &'a mut Component) {
        self.last_id += 1;
        component.set_id(self.last_id);
        component.set_chan(self.msg_chan.clone());
        println!("new component add: {} ({})", component.get_name(), component.get_id());
        self.components.push(component);
    }
    pub fn start(&mut self) {
        let port = self.msg_port.take().unwrap();
        println!("ComponentManager::start()");
        do spawn {
            loop {
                match port.recv() {
                    Message(component_id, MsgPts(pts)) => {
                        println!("msg from {}: MsgPts({})", component_id, pts);
                    }
                }
            }
        }
    }
}
