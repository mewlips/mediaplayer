use avcodec;
use av_stream::AVStream;
use avutil;
use ffmpeg_decoder::FFmpegDecoder;
use std::cast::{transmute_immut_unsafe};
use std::libc::c_int;
use std::ptr::{mut_null,to_mut_unsafe_ptr};
use std::vec;
use component_manager::{Component,ComponentStruct,AudioDecoderComponent,
                        AudioRendererComponent,
                        ManagerComponent,ClockComponent,ExtractorComponent,
                        Message,MsgStart,MsgPts,MsgExtract,MsgPacketData,MsgAudioData};

pub struct AudioData {
    chunk: ~[u8],
    pts: f64,
}

impl AudioData {
    pub fn new(chunk: ~[u8], pts: f64) -> AudioData {
        AudioData {
            chunk: chunk,
            pts: pts,
        }
    }
}

pub struct AudioDecoder {
    component: Option<ComponentStruct>,
    decoder: FFmpegDecoder,
}

impl AudioDecoder {
    pub fn new(audio_stream: &AVStream) -> Option<AudioDecoder> {
        match FFmpegDecoder::new(audio_stream) {
            Some(decoder) => {
                Some(AudioDecoder {
                    component: Some(ComponentStruct::new(AudioDecoderComponent)),
                    decoder: decoder,
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
            println!("sample_fmt = {}, {}", (*codec_ctx).sample_fmt, avutil::AV_SAMPLE_FMT_S16P);
            if (*codec_ctx).sample_fmt == avutil::AV_SAMPLE_FMT_S16P {
                (*codec_ctx).request_sample_fmt = avutil::AV_SAMPLE_FMT_S16;
            }
        }
        let time_base = self.decoder.time_base.clone();
        let component = self.component.take().unwrap();
        do spawn {
            match component.recv() {
                Message { from: ManagerComponent, msg: MsgStart, .. } => {
                    info!("start AudioDecoder");
                    while AudioDecoder::decode(&component, codec_ctx, time_base) {
                        ;
                    }
                }
                _ => {
                    fail!("unexpected message received");
                }
            }
        }
    }
    fn decode(component: &ComponentStruct,
              codec_ctx: *mut avcodec::AVCodecContext,
              time_base: avutil::AVRational) -> bool {
        match component.recv() {
            Message { msg: MsgPacketData(packet), .. } => {
                let mut got_frame: c_int = 0;
                unsafe {
                    let frame = avcodec::avcodec_alloc_frame();
                    avcodec::avcodec_decode_audio4(
                        codec_ctx, frame, to_mut_unsafe_ptr(&mut got_frame),
                        transmute_immut_unsafe(packet));
                    let pts = (*packet).pts as f64 * avutil::av_q2d(time_base);
                    avcodec::av_free_packet(packet);
                    if got_frame != 0 {
                        component.send(ClockComponent, MsgPts(pts.clone()));
                        let data_size = avutil::av_samples_get_buffer_size(
                            mut_null(), (*codec_ctx).channels, (*frame).nb_samples,
                            (*codec_ctx).sample_fmt, 1);
                        component.send(AudioRendererComponent,
                           MsgAudioData(AudioData::new(
                                vec::from_buf::<u8>(
                                    transmute_immut_unsafe((*frame).data[0]),
                                    data_size as uint), pts)));
                    } else {
                        component.send(ExtractorComponent, MsgExtract)
                    }
                }
                true
            }
            _ => {
                // TODO
                false
            }
        }
    }
}

impl Drop for AudioDecoder {
    fn drop(&mut self) {
        debug!("AudioDecoder::drop()");
    }
}

impl Component for AudioDecoder {
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct {
        self.component.get_mut_ref()
    }
}
