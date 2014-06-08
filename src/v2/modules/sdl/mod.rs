use module::Module;
use sdl;

pub struct SdlModule {
    pub name: &'static str,
}

impl Module for SdlModule {
    fn new() -> SdlModule {
        SdlModule {
            name: "SDL",
        }
    }
    fn get_name(&self) -> &'static str {
        self.name
    }
    fn init(&self) -> bool {
        match sdl::init(&[sdl::InitVideo, sdl::InitAudio, sdl::InitTimer]) {
            true => {
                debug!("sdl::init()");
                true
            }
            false => {
                error!("sdl::init() failed");
                false
            }
        }
    }
}

impl Drop for SdlModule {
    fn drop(&mut self) {
        sdl::quit();
        debug!("sdl::quit()");
    }
}
