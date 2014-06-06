use component::{Extractor};
use module::ModuleManager;

pub struct Player<'a> {
    module_manager: &'a ModuleManager,
    source: Option<String>,
    extractor: Option<Box<Extractor>>,
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
        self.source = Some(source.to_owned());

        if self.extractor.is_none() {
            self.extractor = self.module_manager.get_extractor();
        }
        match self.extractor {
            Some(ref mut extractor) => {
                extractor.set_data_source(&Path::new(source.to_owned()));
            }
            None => {
                error!("no extractor found!");
            }
        }
    }
}
