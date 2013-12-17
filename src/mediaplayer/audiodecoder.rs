use avcodec;
use avstream::AVStream;
use avutil;
use ffmpegdecoder::FFmpegDecoder;
use sdl::audio;
use util;
use extra::arc::RWArc;
use extra::dlist::DList;

pub static SDL_AudioBufferSize: u16  = 1024;

mod audio_alt {
    use sdl::audio::{AudioFormat,Channels,ll,ObtainedAudioSpec};
    use avcodec;
    use std::libc::{c_int,c_void};
    use std::ptr::null;
    use std::cast::{forget,transmute};
    use extra::arc::RWArc;
    use extra::dlist::DList;

    pub struct AudioCallback {
        callback: fn(&mut [u8], *mut avcodec::AVCodecContext,
                     RWArc<~DList<*mut avcodec::AVPacket>>),
        codec_ctx: *mut avcodec::AVCodecContext,
        audio_queue: RWArc<~DList<*mut avcodec::AVPacket>>,
    }

    pub struct DesiredAudioSpec {
        freq: c_int,
        format: AudioFormat,
        channels: Channels,
        samples: u16,
        callback: AudioCallback,
    }

    impl DesiredAudioSpec {
        fn to_ll_spec(self) -> ll::SDL_AudioSpec {
            unsafe {
                let DesiredAudioSpec { freq, format, channels, samples, callback } = self;
                ll::SDL_AudioSpec {
                    freq: freq,
                    format: format.to_ll_format(),
                    channels: channels.count() as u8,
                    silence: 0,
                    samples: samples,
                    padding: 0,
                    size: 0,
                    callback: native_callback as *u8,
                    userdata: transmute(~callback),
                }
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
        let cb: ~AudioCallback = unsafe { transmute(userdata) };
        let callback = cb.callback;
        let codec_ctx = cb.codec_ctx;
        let audio_queue = cb.audio_queue;
        let buffer = unsafe { transmute((stream, len as uint)) };
        callback(buffer, codec_ctx, audio_queue);
        unsafe {
            forget(callback);
        }
    }
}

struct AudioDecoder {
    decoder: FFmpegDecoder,
}

impl AudioDecoder {
    pub fn new(audio_stream: &AVStream) -> Option<AudioDecoder> {
        match FFmpegDecoder::new(audio_stream) {
            Some(decoder) => {
                Some(AudioDecoder {
                    decoder: decoder,
                })
            }
            None => {
                None
            }
        }
    }
    pub fn audio_callback(buffer: &mut [u8],
                          codec_ctx: *mut avcodec::AVCodecContext,
                          audio_queue: RWArc<~DList<*mut avcodec::AVPacket>>) {
        loop {
            audio_queue.read(|queue| {
                for packet in queue.move_iter() {
                    println!("audio_callback {} {} {}", buffer.len(), codec_ctx,
                             packet);
                }
            });
        }

        for u in buffer.mut_iter() {
            *u = 128;
        }
        //util::usleep(100_000);

        /*match ad_port.recv() {
            Some(packet) => {
                AudioDecoder::decode(buffer, codec_ctx, packet);
            }
            None => {
            }
        }*/
    }
    pub fn start(&self, audio_queue: RWArc<~DList<*mut avcodec::AVPacket>>) {
        let codec_ctx = self.decoder.codec_ctx.clone();

        let wanted_spec = audio_alt::DesiredAudioSpec {
            freq: unsafe { (*codec_ctx).sample_rate },
            format: audio::S16_AUDIO_FORMAT,
            channels: audio::Channels::new(unsafe { (*codec_ctx).channels }),
            samples: SDL_AudioBufferSize,
            callback: audio_alt::AudioCallback {
                          callback: AudioDecoder::audio_callback,
                          codec_ctx: codec_ctx,
                          audio_queue: audio_queue,
                      }
        };

        match audio_alt::open(wanted_spec) {
            Ok(_obtained_spec) => {
            }
            Err(_) => {
                error!("audio open failed()");
            }
        }

        //audio::pause(false);
    }
    fn decode(buffer: &mut [u8],
              codec_ctx: *mut avcodec::AVCodecContext,
              packet: *mut avcodec::AVPacket) {
        println("decode");
    }
}
