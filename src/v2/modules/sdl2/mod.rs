use module::Module;

pub struct Sdl2Module {
    pub name: &'static str,
}

impl Module for Sdl2Module {
    fn new() -> Sdl2Module {
        Sdl2Module {
            name: "SDL"
        }
    }
    fn get_name(&self) -> &'static str {
        self.name
    }
    fn init(&self) -> bool {
        true
    }
}

impl Drop for Sdl2Module {
    fn drop(&mut self) {
        debug!("sdl::quit()\n");
    }
}
