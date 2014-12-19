use avcodec;
use av_stream::AVStream;
use avutil;
use ffmpeg_decoder::FFmpegDecoder;
use std::c_vec::CVec;
use std::mem::{transmute};
use libc::c_int;
use std::ptr::{null_mut};
use component::{Component,ComponentStruct};
use component::ComponentType::{AudioDecoderComponent,AudioRendererComponent,
                               ClockComponent,ExtractorComponent};
use message::{Message,MessageData};
use message::MessageData::{MsgStop,MsgPts,MsgExtract,
                           MsgPacketData,MsgAudioData,MsgFlush};
use swresample;
use util;

#[deriving(Clone)]
pub struct AudioData {
    pub chunk: Vec<u8>,
    pub pts: f64,
}

impl AudioData {
    pub fn new(chunk: Vec<u8>, pts: f64) -> AudioData {
        AudioData {
            chunk: chunk,
            pts: pts,
        }
    }
}

pub struct AudioDecoder {
    pub component: Option<ComponentStruct>,
    pub decoder: FFmpegDecoder,
    pub swr_ctx: Option<*mut swresample::SwrContext>,
}

impl AudioDecoder {
    pub fn new(audio_stream: &AVStream) -> Option<AudioDecoder> {
        match FFmpegDecoder::new(audio_stream) {
            Some(decoder) => {
                let codec = unsafe { &mut (*(*audio_stream.av_stream).codec) };
                let swr_ctx: Option<*mut swresample::SwrContext> =
                    if codec.sample_fmt != avutil::AV_SAMPLE_FMT_S16 {
                        info!("audio need to resample. (libswresample)");
                        let swr_ctx = unsafe { swresample::swr_alloc() };
                        match (codec.channel_layout, codec.channels) {
                            (0, 1) => {
                                codec.channel_layout = avutil::AV_CH_LAYOUT_MONO;
                            }
                            (0, c) if c == 1 || c == 2 => {
                                codec.channel_layout = avutil::AV_CH_LAYOUT_STEREO;
                            }
                            _ => {}
                        }
                        unsafe {
                            "in_channel_layout".with_c_str(|name| {
                                avutil::av_opt_set_int(transmute(swr_ctx), name,
                                    codec.channel_layout as i64, 0);
                            });
                            "in_sample_fmt".with_c_str(|name| {
                                avutil::av_opt_set_int(transmute(swr_ctx), name,
                                    codec.sample_fmt as i64, 0);
                            });
                            "in_sample_rate".with_c_str(|name| {
                                avutil::av_opt_set_int(transmute(swr_ctx), name,
                                    codec.sample_rate as i64, 0);
                            });
                            "out_channel_layout".with_c_str(|name| {
                                avutil::av_opt_set_int(transmute(swr_ctx), name,
                                    avutil::AV_CH_LAYOUT_STEREO as i64, 0);
                            });
                            "out_sample_fmt".with_c_str(|name| {
                                avutil::av_opt_set_int(transmute(swr_ctx), name,
                                    avutil::AV_SAMPLE_FMT_S16 as i64, 0);
                            });
                            "out_sample_rate".with_c_str(|name| {
                                avutil::av_opt_set_int(transmute(swr_ctx), name,
                                    codec.sample_rate as i64, 0);
                            });
                        }
                        if unsafe { swresample::swr_init(transmute(swr_ctx)) } < 0 {
                            error!("swr_init() failed");
                            None
                        } else {
                            Some(swr_ctx)
                        }

                    } else {
                        None
                    };

                Some(AudioDecoder {
                    component: Some(ComponentStruct::new(AudioDecoderComponent)),
                    decoder: decoder,
                    swr_ctx: swr_ctx,
                })
            }
            None => {
                None
            }
        }
    }
    pub fn start(&mut self) {
        let codec_ctx = self.decoder.codec_ctx.clone();
        unsafe {
            if (*codec_ctx).sample_fmt == avutil::AV_SAMPLE_FMT_S16P {
                (*codec_ctx).request_sample_fmt = avutil::AV_SAMPLE_FMT_S16;
            }
        }
        let time_base = self.decoder.time_base.clone();
        let component = self.component.take().unwrap();
        let swr_ctx = self.swr_ctx.clone();
        spawn(move || {
            component.wait_for_start();
            while AudioDecoder::decode(&component, codec_ctx,
                                       time_base.clone(), swr_ctx) {
                ;
            }
            info!("stop AudioDecoder");
        })
    }
    fn decode(component: &ComponentStruct,
              codec_ctx: *mut avcodec::AVCodecContext,
              time_base: avutil::AVRational,
              swr_ctx: Option<*mut swresample::SwrContext>) -> bool {
        match component.recv() {
            Message { msg: MsgPacketData(packet), .. } => {
                let mut got_frame: c_int = 0;
                unsafe {
                    let frame = avcodec::avcodec_alloc_frame();
                    avcodec::avcodec_decode_audio4(
                        codec_ctx, frame, &mut got_frame, &*packet);
                    let pts = (*packet).pts as f64 * avutil::av_q2d(time_base);
                    avcodec::av_free_packet(packet);
                    if got_frame != 0 {
                        component.send(ClockComponent, MsgPts(pts.clone()));
                        let data_size = avutil::av_samples_get_buffer_size(
                            null_mut(), (*codec_ctx).channels, (*frame).nb_samples,
                            (*codec_ctx).sample_fmt, 1);
                        match swr_ctx {
                            Some(swr_ctx) => {
                                match AudioDecoder::resample(
                                    &mut (*swr_ctx), &mut (*frame)) {
                                    Some(chunk) => {
                                        component.send(AudioRendererComponent,
                                            MsgAudioData(AudioData::new(
                                                chunk, pts)));
                                    }
                                    None => {}
                                }
                            }
                            None => {
                                let cv = CVec::<u8>::new(transmute((*frame).data[0]),
                                                        data_size as uint);
                                let v = cv.as_slice().to_vec();
                                component.send(AudioRendererComponent,
                                    MsgAudioData(AudioData::new(v, pts)));
                            }
                        }
                    } else {
                        component.send(ExtractorComponent, MsgExtract);
                    }
                    avcodec::avcodec_free_frame(transmute(&frame));
                }
                true
            }
            Message { msg: MsgStop, .. } => {
                component.flush();
                false
            }
            Message { msg: MsgFlush, .. } => {
                component.flush();
                unsafe {
                    avcodec::avcodec_flush_buffers(codec_ctx);
                }
                true
            }
            _ => {
                error!("unexpected message received");
                false
            }
        }
    }
    fn resample(swr_ctx: &mut swresample::SwrContext,
                frame: &mut avutil::AVFrame) -> Option<Vec<u8>> {
        let mut resampled_out: *mut u8 = null_mut();
        let mut resample_lines: c_int = 0;
        let resample_size: i64 = unsafe {
            avutil::av_rescale_rnd(
                swresample::swr_get_delay(swr_ctx, 44100)
                    + frame.nb_samples as i64,
                44100, 44100, avutil::AV_ROUND_UP)
        };
        let result = unsafe { avutil::av_samples_alloc(
            &mut resampled_out, &mut resample_lines, 2,
            resample_size as i32, avutil::AV_SAMPLE_FMT_S16, 0)
        };
        if result < 0 {
            error!("av_samples_alloc() failed! {}", util::av_strerror(result));
            return None;
        }

        let resample_input = unsafe { &mut (*frame.extended_data) };

        let out_size = unsafe { swresample::swr_convert(
            swr_ctx, &mut resampled_out, resample_size as i32,
            transmute(resample_input), frame.nb_samples)
        };
        let out_bytes = unsafe { avutil::av_samples_get_buffer_size(
            null_mut(), 2, out_size, avutil::AV_SAMPLE_FMT_S16, 1)
        };

        if out_size < 0 {
            error!("resample failed");
            return None;
        }

        let resampled = unsafe {
            let cv = CVec::<u8>::new(transmute(resampled_out), out_bytes as uint);
            cv.as_slice().to_vec()
        };

        unsafe {
            avutil::av_free(transmute(resampled_out));
        }

        Some(resampled)
    }
}

impl Drop for AudioDecoder {
    fn drop(&mut self) {
        debug!("AudioDecoder::drop()");
    }
}

impl Component for AudioDecoder {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct {
        self.component.as_mut().unwrap()
    }
}
