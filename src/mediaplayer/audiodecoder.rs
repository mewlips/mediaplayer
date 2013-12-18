use avcodec;
use avstream::AVStream;
use avutil;
use ffmpegdecoder::FFmpegDecoder;
use sdl::audio;
use util;
use buffer::AudioBuffer;
use std::cast::transmute;
use extra::arc::RWArc;
use std::libc::c_int;
use std::ptr::{mut_null,to_mut_unsafe_ptr};
use std::cast::{transmute_immut_unsafe};

pub static SDL_AudioBufferSize: u16  = 4096;

mod audio_alt {
    use sdl::audio::{AudioFormat,Channels,ll,ObtainedAudioSpec};
    use avcodec;
    use std::libc::{c_int,c_void};
    use std::ptr::null;
    use std::cast::{transmute};
    use buffer::AudioBuffer;
    use std::cast::forget;

    pub struct DesiredAudioSpec {
        freq: c_int,
        format: AudioFormat,
        channels: Channels,
        samples: u16,
        callback: *c_void,
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
        println!("native_callback {}", len);
        let audio_buffer: ~AudioBuffer = unsafe { transmute(userdata) };
        let mut idx = 0;
        while idx < len && idx < (audio_buffer.data.len() as i32) {
            unsafe {
                *stream.offset(idx as int) = audio_buffer.data[idx];
            }
            idx += 1;
        }
        unsafe {
            forget(audio_buffer);
        }

        //::util::usleep(10_1000);
        //let buffer: &mut [u8] = unsafe { transmute((stream, len as uint)) };
        //for i in buffer.mut_iter() {
        //    *i = 128;
        //}
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
    pub fn start(&self, ad_port: Port<Option<*mut avcodec::AVPacket>>) {
        let codec_ctx = self.decoder.codec_ctx.clone();

        let audio_buffer = RWArc::new(AudioBuffer::new());

        let wanted_spec = audio_buffer.read(|audio_buffer| {
            audio_alt::DesiredAudioSpec {
                freq: unsafe { (*codec_ctx).sample_rate },
                format: audio::S16_AUDIO_FORMAT,
                channels: audio::Channels::new(unsafe { (*codec_ctx).channels }),
                samples: SDL_AudioBufferSize,
                callback: unsafe { transmute(&audio_buffer) },
            }
        });

        match audio_alt::open(wanted_spec) {
            Ok(_obtained_spec) => {
            }
            Err(_) => {
                error!("audio open failed()");
            }
        }

        audio::pause(false);

        do spawn {
            while AudioDecoder::decode(codec_ctx, &ad_port, &audio_buffer) {
                ;
            }
        }
    }
    fn decode(codec_ctx: *mut avcodec::AVCodecContext,
              ad_port: &Port<Option<*mut avcodec::AVPacket>>,
              audio_buffer: &RWArc<AudioBuffer>) -> bool {
        match ad_port.recv() {
            Some(packet) => {
                println!("decode");
                let mut got_frame: c_int = 0;
                unsafe {
                    let frame = avcodec::avcodec_alloc_frame();
                    avcodec::avcodec_decode_audio4(
                        codec_ctx, frame, to_mut_unsafe_ptr(&mut got_frame),
                        transmute_immut_unsafe(packet));
                    avcodec::av_free_packet(packet);
                    if got_frame != 0 {
                        let data_size = avutil::av_samples_get_buffer_size(
                            mut_null(), (*codec_ctx).channels, (*frame).nb_samples,
                            (*codec_ctx).sample_fmt, 1);
                        println!("data_size = {}", data_size);
                        audio_buffer.write(|audio_buffer| {
                            audio_buffer.copy((*frame).data[0], data_size as int);
                        });
                    }
                }
                true
            }
            None => {
                info!("null packet received");
                audio::pause(true);
                false
            }
        }
    }
}
