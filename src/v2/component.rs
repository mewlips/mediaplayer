use stream::{Stream};
use std::comm::{sync_channel, SyncSender};

pub enum Message {
    MsgAddSender(SyncSender<Message>),
}

pub struct Pipe {
    pub receivers: Vec<Receiver<Message>>,
    pub senders: Vec<SyncSender<Message>>,
}

impl Pipe {
    pub fn new() -> Pipe {
        Pipe {
            receivers: Vec::new(),
            senders: Vec::new(),
        }
    }
    pub fn get_sender(&mut self, bound: uint) -> SyncSender<Message> {
        let (sender, receiver) = sync_channel::<Message>(bound);
        self.receivers.push(receiver);
        sender
    }
    pub fn get_receiver(&mut self, bound: uint) -> Receiver<Message> {
        let (sender, receiver) = sync_channel::<Message>(bound);
        self.senders.push(sender);
        receiver
    }
    pub fn add_sender(&mut self, sender: SyncSender<Message>) {
        self.senders.push(sender);
    }
    pub fn add_receiver(&mut self, receiver: Receiver<Message>) {
        self.receivers.push(receiver);
    }
}
