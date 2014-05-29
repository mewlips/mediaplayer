use component::{Extractor};

pub trait Module {
    fn new() -> Self;
    fn get_name(&self) -> &'static str;
    fn init(&self) -> bool;
    fn get_extractor(&self) -> Option<Box<Extractor>> {
        None
    }

}

pub struct ModuleManager {
    modules: Vec<Box<Module>>,
}

impl ModuleManager {
    pub fn new() -> ModuleManager {
        ModuleManager {
            modules: Vec::new(),
        }
    }

    pub fn add(&mut self, module: Box<Module>) {
        self.modules.push(module);
    }
    pub fn init(&self) -> bool {
        self.modules.iter().all(|m| m.init())
    }
    pub fn get_extractor(&self) -> Option<Box<Extractor>> {
        for module in self.modules.iter() {
            match module.get_extractor() {
                Some(extractor) => {
                    debug!("found {}.", extractor.get_name());
                    return Some(extractor);
                }
                None => {
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use module::{Module,ModuleManager};
    use module::tests::dummy::DummyModule;

    mod dummy {
        use module::Module;

        pub struct DummyModule;

        impl Module for DummyModule {
            fn new() -> DummyModule {
                DummyModule
            }
            fn get_name(&self) -> &'static str {
                "Dummy"
            }
            fn init(&self) -> bool {
                true
            }
        }
    }

    #[test]
    fn test_module_manager() {
        let module_manager = ModuleManager::new();
        assert!(module_manager.init());
    }

    #[test]
    fn test_module_manager_add() {
        let mut module_manager = ModuleManager::new();
        let module: Box<DummyModule> = box Module::new();
        assert!(module.get_name() == "Dummy".as_slice());
        module_manager.add(module);
    }

}
