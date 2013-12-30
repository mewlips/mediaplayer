extern mod avcodec;
extern mod avformat;
extern mod avutil;
extern mod extra;
extern mod sdl;
extern mod swscale;

use avformat::av_register_all;
use extra::getopts::{getopts,optflag,Opt};
use extra::url;
use mediaplayer::MediaPlayer;
use std::libc::consts::os::c95::EXIT_FAILURE;
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

pub fn init() -> bool {
    unsafe {
        av_register_all();
        debug!("av_register_all()");
    }
    match sdl::init(&[sdl::InitVideo, sdl::InitAudio, sdl::InitTimer]) {
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
    let program = args[0].clone();
    let opts = ~[
        optflag("h"),
        optflag("help")
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => {
            error!("{}\n", f.to_err_msg());
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
}

pub fn print_usage(program: &str, _opts: &[Opt]) {
    println!("Usage: {} [options] <files>...", program);
    println("\n[options]");
    println("  -h, --help\t: show usage.");
}

pub fn play(source: ~str) -> bool {
    let mut mp = MediaPlayer::new();
    match url::from_str(source) {
        Ok(url) => {
            mp.set_url_source(url);
        }
        Err(_) => {
            let path = Path::new(source);
            if path.exists() {
                mp.set_file_source(path);
            }
        }
    }
    if !mp.prepare() {
        return false;
    }

    mp.start();

    'main: loop {
        match sdl::event::poll_event() {
            sdl::event::QuitEvent => {
                sdl::quit();
                break;
            }
            /*sdl::event::MouseButtonEvent(button, state, _, _) => {
                match button {
                    sdl::event::LeftMouse if state => {
                        mp.send_cmd(mediaplayer::Start);
                    }
                    _ => {
                    }
                }
            }*/
            sdl::event::NoEvent => {
            }
            _ => {}
        }
        util::usleep(33_000);
    }

    true
}
