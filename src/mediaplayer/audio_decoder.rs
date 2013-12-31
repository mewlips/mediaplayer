use avcodec;
use av_stream::AVStream;
use avutil;
use ffmpeg_decoder::FFmpegDecoder;
use std::cast::{transmute_immut_unsafe};
use std::libc::c_int;
use std::ptr::{mut_null,to_mut_unsafe_ptr};
use std::vec;
use component_manager::{Component,ComponentId,Message,MsgPts};

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
    component_id: Option<ComponentId>,
    chan: Option<SharedChan<Message>>,
    decoder: FFmpegDecoder,
}

impl AudioDecoder {
    pub fn new(audio_stream: &AVStream) -> Option<AudioDecoder> {
        match FFmpegDecoder::new(audio_stream) {
            Some(decoder) => {
                Some(AudioDecoder {
                    component_id: None,
                    chan: None,
                    decoder: decoder,
                })
            }
            None => {
                None
            }
        }
    }
    pub fn start(&mut self, ad_port: Port<Option<*mut avcodec::AVPacket>>,
                            ar_chan: Chan<Option<~AudioData>>) {
        let codec_ctx = self.decoder.codec_ctx.clone();
        unsafe {
            println!("sample_fmt = {}, {}", (*codec_ctx).sample_fmt, avutil::AV_SAMPLE_FMT_S16P);
            if (*codec_ctx).sample_fmt == avutil::AV_SAMPLE_FMT_S16P {
                (*codec_ctx).request_sample_fmt = avutil::AV_SAMPLE_FMT_S16;
            }
        }
        let time_base = self.decoder.time_base.clone();

        let component_id = self.get_id().unwrap();
        let chan = self.chan.take().unwrap();
        do spawn {
            while AudioDecoder::decode(component_id, &chan,
                                       codec_ctx, time_base,
                                       &ad_port, &ar_chan) {
                ;
            }
        }
    }
    fn decode(component_id: ComponentId, chan: &SharedChan<Message>,
              codec_ctx: *mut avcodec::AVCodecContext,
              time_base: avutil::AVRational,
              ad_port: &Port<Option<*mut avcodec::AVPacket>>,
              ar_chan: &Chan<Option<~AudioData>>) -> bool {
        match ad_port.recv() {
            Some(packet) => {
                let mut got_frame: c_int = 0;
                unsafe {
                    let frame = avcodec::avcodec_alloc_frame();
                    avcodec::avcodec_decode_audio4(
                        codec_ctx, frame, to_mut_unsafe_ptr(&mut got_frame),
                        transmute_immut_unsafe(packet));
                    let pts = (*packet).pts as f64 * avutil::av_q2d(time_base);
                    chan.send(Message(component_id.clone(), MsgPts(pts.clone())));
                    avcodec::av_free_packet(packet);
                    if got_frame != 0 {
                        let data_size = avutil::av_samples_get_buffer_size(
                            mut_null(), (*codec_ctx).channels, (*frame).nb_samples,
                            (*codec_ctx).sample_fmt, 1);
                        ar_chan.send(Some(~AudioData::new(
                            vec::from_buf::<u8>(
                                transmute_immut_unsafe((*frame).data[0]),
                                data_size as uint), pts)));
                    }
                }
                true
            }
            None => {
                info!("null packet received");
                ar_chan.send(None);
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
    fn set_id(&mut self, id: ComponentId) {
        self.component_id = Some(id);
    }
    fn get_id(&self) -> Option<ComponentId> {
        self.component_id
    }
    fn get_name(&self) -> &str {
        "AudioDecoder"
    }
    fn set_chan(&mut self, chan: SharedChan<Message>) {
        self.chan = Some(chan);
    }
}

