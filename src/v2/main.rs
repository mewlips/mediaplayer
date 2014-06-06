#![feature(phase)]

extern crate getopts;
extern crate libc;
#[phase(syntax, link)]     extern crate log;

#[cfg(ffmpeg, avcodec55)]  extern crate ll_avcodec = "avcodec55";
#[cfg(ffmpeg, avformat55)] extern crate ll_avformat = "avformat55";
#[cfg(ffmpeg)]             extern crate ll_avutil = "avutil52";
#[cfg(ffmpeg)]             extern crate ll_swscale = "swscale2";
#[cfg(ffmpeg)]             extern crate ll_swresample = "swresample0";
#[cfg(sdl)]                extern crate sdl;

use getopts::{getopts,optflag,OptGroup};
use module::{Module,ModuleManager};
use std::os;

#[cfg(sdl)]    use modules::sdl::SdlModule;
#[cfg(sdl2)]   use modules::sdl2::Sdl2Module;
#[cfg(ffmpeg)] use modules::ffmpeg::FFmpegModule;
use player::Player;

mod module;
mod modules;
mod player;
mod component;
mod stream;

pub fn main() {
    let args = os::args();
    let program = args.get(0).as_slice();
    let opts = ~[
        optflag("h", "help", "show help"),
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => {
            error!("{}\n", f.to_err_msg());
            print_usage(program, opts);
            //os::set_exit_status(EXIT_FAILURE as int);
            return;
        }
    };
    if matches.opt_present("h") || matches.opt_present("help") {
        print_usage(program, opts);
        return;
    }

    let mut module_manager = ModuleManager::new();

    let module: Box<SdlModule> = box Module::new();
    module_manager.add(module as Box<Module>);

    let module: Box<FFmpegModule> = box Module::new();
    module_manager.add(module as Box<Module>);

    module_manager.init();

    let mut player = Player::init(&module_manager);

    let sources = matches.free;
    for source in sources.iter() {
        player.play(source);
    }
}

fn print_usage(program: &str, _opts: &[OptGroup]) {
    println!("Usage: {} [options] <files>...", program);
    println!("\n[options]");
    println!("  -h, --help\t: show usage.");
}
