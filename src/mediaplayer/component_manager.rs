use component::{ComponentType,Component,ManagerComponent};
use message::{Message,MessageData,MsgStart,MsgEOF,MsgError,MsgStop};

pub struct ComponentManager {
    priv mp_chan: Option<Chan<bool>>,
    priv components: Option<~[(ComponentType, Chan<Message>)]>,
    priv msg_port: Option<Port<Message>>,
    priv msg_chan: SharedChan<Message>,
}

impl ComponentManager {
    pub fn new(mp_chan: Chan<bool>) -> ComponentManager {
        let (port, chan) = SharedChan::<Message>::new();
        ComponentManager {
            mp_chan: Some(mp_chan),
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
        let mp_chan = self.mp_chan.take().unwrap();
        do spawn {
            let broadcast = |msg: MessageData| {
                for &(component_type, ref chan) in components.iter() {
                    chan.send(Message::new(ManagerComponent, component_type,
                                           msg.clone()));
                }
            };
            broadcast(MsgStart);
            loop {
                match port.recv() {
                    Message { from, to, msg } => {
                        //debug!("from = {}, to = {}, msg = {}", from, to, msg);
                        if to == ManagerComponent {
                            match msg {
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
                                    chan.send(Message::new(from, to, msg));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            debug!("stop ComponentManager");
            mp_chan.send(true);
        }
    }
}
