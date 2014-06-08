use component::{Message,Pipe,MsgAddSender};
use module::ModuleManager;
use std::comm::{sync_channel, SyncSender};

pub struct Player<'a> {
    module_manager: &'a ModuleManager,
    source: Option<String>,
    extractor: Option<SyncSender<Message>>,
}

impl<'a> Player<'a> {
    pub fn init<'a>(module_manager: &'a ModuleManager) -> Player<'a> {
        Player {
            module_manager: module_manager,
            source: None,
            extractor: None,
        }
    }
    pub fn play(&mut self, source: &String) {
        let (my_sender, receiver) = sync_channel::<Message>(100);
        self.source = Some(source.to_owned());

        let extractor = self.module_manager.get_extractor();
        match extractor {
            Some((sender, procedure)) => {
                self.extractor = Some(sender.clone());
                spawn(procedure);
                sender.send(MsgAddSender(my_sender.clone()));
            }
            None => {
                error!("no extractor found");
            }
        }
    }
}
