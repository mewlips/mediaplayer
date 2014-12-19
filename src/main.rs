#![feature(phase)]

extern crate "avcodec55" as avcodec;
extern crate "avformat55" as avformat;
extern crate "avutil52" as avutil;
extern crate getopts;
extern crate libc;
#[phase(plugin, link)]
extern crate log;
extern crate sdl;
extern crate "swscale2" as swscale;
extern crate "swresample0" as swresample;

use avformat::av_register_all;
use getopts::{getopts,optflag,OptGroup};
use mediaplayer::MediaPlayer;
use libc::consts::os::c95::EXIT_FAILURE;
use std::io::fs::PathExtensions;
use std::os;

mod av_stream;
mod extractor;
mod mediaplayer;
mod util;
mod ffmpeg_decoder;
mod video_decoder;
mod audio_decoder;
mod video_renderer;
mod audio_renderer;
mod audio_pipe;
mod clock;
mod component_manager;
mod ui;
mod component;
mod message;

pub fn init() -> bool {
    unsafe {
        av_register_all();
        debug!("av_register_all()");
    }
    match sdl::init(&[sdl::InitFlag::Video, sdl::InitFlag::Audio, sdl::InitFlag::Timer]) {
        true =>  {
            debug!("sdl::init()");
            true
        }
        false => {
            os::set_exit_status(EXIT_FAILURE as int);
            error!("sdl::init() failed");
            false
        }
    }
}

pub fn main() {
    let args = os::args();
    let program = args.get(0).unwrap().clone();
    let opts = &[
        optflag("h", "help", "show help"),
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => {
            error!("{}\n", f);
            print_usage(program, opts);
            os::set_exit_status(EXIT_FAILURE as int);
            return;
        }
    };
    if matches.opt_present("h") || matches.opt_present("help") {
        print_usage(program, opts);
        return;
    }

    init();

    let sources = matches.free;
    for source in sources.iter() {
        play(source.clone());
    }
    sdl::quit();
}

pub fn print_usage(program: String, _opts: &[OptGroup]) {
    println!("Usage: {} [options] <files>...", program);
    println!("\n[options]");
    println!("  -h, --help\t: show usage.");
}

pub fn play(source: String) -> bool {
    let mut mp = MediaPlayer::new();
    let path = Path::new(source);
    if path.exists() {
        mp.set_file_source(path);
    }
    if !mp.prepare() {
        return false;
    }

    mp.start();

    mp.wait();

    true
}
