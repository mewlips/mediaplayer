use std::libc::c_int;
use std::os;
use sdl::audio;
use audio_pipe::AudioPipe;
use avcodec;
use std::cast::transmute;
use std::libc;
use audio_decoder::AudioData;

pub static SDL_AudioBufferSize: u16 = 1024;

mod audio_alt {
    use sdl::audio::{AudioFormat,Channels,ll,ObtainedAudioSpec};
    use std::libc::{c_int,c_void};
    use std::ptr::null;
    use std::cast::{transmute};
    use audio_pipe::AudioPipe;
    use std::cast::forget;

    pub struct DesiredAudioSpec {
        freq: c_int,
        format: AudioFormat,
        channels: Channels,
        samples: u16,
        userdata: *c_void,
    }

    impl DesiredAudioSpec {
        fn to_ll_spec(self) -> ll::SDL_AudioSpec {
            let DesiredAudioSpec { freq, format, channels, samples, userdata} = self;
            ll::SDL_AudioSpec {
                freq: freq,
                format: format.to_ll_format(),
                channels: channels.count() as u8,
                silence: 0,
                samples: samples,
                padding: 0,
                size: 0,
                callback: native_callback as *u8,
                userdata: userdata,
            }
        }
    }

    pub fn open(desired: DesiredAudioSpec) -> Result<ObtainedAudioSpec,()> {
        unsafe {
            let mut ll_desired = desired.to_ll_spec();
            let mut ll_obtained = ll::SDL_AudioSpec {
                freq: 0,
                format: 0,
                channels: 0,
                silence: 0,
                samples: 0,
                padding: 0,
                size: 0,
                callback: null(),
                userdata: null(),
            };

            if ll::SDL_OpenAudio(&mut ll_desired, &mut ll_obtained) < 0 {
                Err(())
            } else {
                Ok(ObtainedAudioSpec {
                    freq: ll_obtained.freq,
                    format: AudioFormat::from_ll_format(ll_obtained.format),
                    channels: Channels::new(ll_obtained.channels as c_int),
                    silence: ll_obtained.silence,
                    samples: ll_obtained.samples,
                    size: ll_obtained.size,
                })
            }
        }
    }

    extern fn native_callback(userdata: *c_void, stream: *mut u8, len: c_int) {
        unsafe {
            let mut audio_pipe: ~AudioPipe = transmute(userdata);
            audio_pipe.copy_to(stream, len as uint);
            forget(audio_pipe);
        }
    }
}


pub struct AudioRenderer {
    codec_ctx: *mut avcodec::AVCodecContext,
    pipe_out: c_int,
    audio_pipe: AudioPipe,
}

impl AudioRenderer {
    pub fn new(codec_ctx: *mut avcodec::AVCodecContext) -> Option<AudioRenderer> {
        let os::Pipe {input: pipe_input, out: pipe_out} = os::pipe();
        println!("pipe_input = {}, pipe_out = {}", pipe_input, pipe_out);

        let audio_pipe = AudioPipe::new(pipe_input);
        Some(AudioRenderer {
            codec_ctx: codec_ctx.clone(),
            pipe_out: pipe_out,
            audio_pipe: audio_pipe,
        })
    }
    pub fn start(&self, ar_port: Port<Option<~AudioData>>, as_chan: Chan<f64>) {
        let wanted_spec =
            audio_alt::DesiredAudioSpec {
                freq: unsafe { (*self.codec_ctx).sample_rate },
                format: audio::S16_AUDIO_FORMAT,
                channels: audio::Channels::new(unsafe { (*self.codec_ctx).channels }),
                samples: SDL_AudioBufferSize,
                userdata: unsafe { transmute(&self.audio_pipe) },
            };

        match audio_alt::open(wanted_spec) {
            Ok(_obtained_spec) => {
            }
            Err(_) => {
                error!("audio open failed()");
                return;
            }
        }

        let pipe_out = self.pipe_out.clone();
        do spawn {
            let mut paused = true;
            loop {
                let data = ar_port.recv();
                match data {
                    Some(ref data) => {
                        let ptr = unsafe { transmute(data.chunk.as_ptr()) };
                        let len = data.chunk.len() as u64;
                        as_chan.send(data.pts.clone());
                        println!("audio pts = {}", data.pts);
                        let result = unsafe {
                            libc::funcs::posix88::unistd::write(
                                pipe_out, ptr, len)
                        };
                        if result >= 0 {
                            if paused {
                                audio::pause(false);
                                paused = false;
                            }
                        } else {
                            error!("write failed!");
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
}

impl Drop for AudioRenderer {
    fn drop(&mut self) {
        debug!("AudioRenderer::drop()");
    }
}
