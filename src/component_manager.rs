use component::{ComponentType,Component,ManagerComponent};
use message::{Message,MessageData,MsgStart,MsgEOF,MsgError,MsgStop};

pub struct ComponentManager {
    mp_sender: Option<Sender<bool>>,
    components: Option<Vec<(ComponentType, Sender<Message>)>>,
    msg_receiver: Option<Receiver<Message>>,
    msg_sender: Sender<Message>,
}

impl ComponentManager {
    pub fn new(mp_sender: Sender<bool>) -> ComponentManager {
        let (sender, receiver) = channel::<Message>();
        ComponentManager {
            mp_sender: Some(mp_sender),
            components: Some(vec!()),
            msg_receiver: Some(receiver),
            msg_sender: sender,
        }
    }
    pub fn add(&mut self, component: &mut Component) {
        let component = component.get();
        let component_type = component.component_type;
        let sender = component.take_sender();
        component.set_mgr_sender(self.msg_sender.clone());
        self.components.get_mut_ref().push((component_type, sender));
        info!("new component add: {}", component.component_type);
    }
    pub fn start(&mut self) {
        let receiver = self.msg_receiver.take().unwrap();
        debug!("ComponentManager::start()");
        let components = self.components.take().unwrap();
        let mp_sender= self.mp_sender.take().unwrap();
        spawn(proc() {
            let broadcast = |msg: MessageData| {
                for &(component_type, ref sender) in components.iter() {
                    sender.send(Message::new(ManagerComponent, component_type,
                                           msg.clone()));
                }
            };
            broadcast(MsgStart);
            loop {
                match receiver.recv() {
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
                            for &(component_type, ref sender) in components.iter() {
                                if component_type == to {
                                    sender.send(Message::new(from, to, msg));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            debug!("stop ComponentManager");
            mp_sender.send(true);
        })
    }
}
