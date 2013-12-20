use avcodec;
use avutil;
use sdl;
use std::cast::{transmute};
use std::libc::{c_int};
use std::ptr::{null,mut_null};
use swscale;
use util;

struct VideoRenderer {
    width: int,
    height: int,
    pix_fmt: avutil::Enum_AVPixelFormat,
}

impl VideoRenderer {
    pub fn new(width: int, height: int, pix_fmt: avutil::Enum_AVPixelFormat)
            -> VideoRenderer {
        debug!("width = {}, height = {}, pix_fmt = {}", width, height, pix_fmt as int);
        VideoRenderer {
            width: width,
            height: height,
            pix_fmt: pix_fmt,
        }
    }
    pub fn start(&self, vr_port: Port<Option<*mut avcodec::AVFrame>>) {
        let screen = match sdl::video::set_video_mode(
                                            self.width, self.height, 24,
                                            [sdl::video::HWSurface],
                                            [sdl::video::DoubleBuf]) {
            Ok(screen) => screen,
            Err(err) => fail!("sdl::video::set_video_mode() failed! {}", err)
        };
        let frame_rgb = unsafe {
            avcodec::avcodec_alloc_frame()
        };
        let sws_ctx = unsafe {
            swscale::sws_getCachedContext(
                mut_null(), self.width as c_int, self.height as c_int, self.pix_fmt,
                self.width as c_int, self.height as c_int, avutil::AV_PIX_FMT_BGR24,
                swscale::SWS_BILINEAR as c_int, mut_null(), mut_null(),
                null())
        };
        let width = self.width.clone();
        let height = self.height.clone();
        do spawn {
            while VideoRenderer::render(width, height, screen, frame_rgb.clone(),
                                        sws_ctx.clone(), &vr_port) {
                ;
            }
        }
    }

    fn render(width: int, height: int,
              screen: &sdl::video::Surface,
              frame_rgb: *mut avcodec::AVFrame,
              sws_ctx: *mut swscale::Struct_SwsContext,
              vr_port: &Port<Option<*mut avcodec::AVFrame>>) -> bool {
        match vr_port.recv() {
            Some(mut frame) => {
                //debug!("frame = {}", frame);
                screen.with_lock(|pixels| {
                    pixels.as_mut_buf(|p, _len| {
                        unsafe {
                            avcodec::avpicture_fill(transmute(frame_rgb),
                                                    transmute(p), avutil::AV_PIX_FMT_BGR24,
                                                    width as c_int, height as c_int);
                            swscale::sws_scale(sws_ctx, transmute((*frame).data.as_ptr()),
                                               (*frame).linesize.as_ptr(), 0, (*frame).height,
                                               transmute((*frame_rgb).data.as_mut_ptr()),
                                               (*frame_rgb).linesize.as_ptr());
                        }
                    });
                });
                screen.flip();
                unsafe {
                    avcodec::avcodec_free_frame(&mut frame);
                }
                true
            }
            None => {
                info!("null frame received")
                false
            }
        }
    }
}

impl Drop for VideoRenderer {
    fn drop(&mut self) {
        debug!("VideoRenderer::drop()");
    }
}
