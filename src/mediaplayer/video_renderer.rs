use avcodec;
use avutil;
use sdl;
use std::cast::{transmute};
use std::libc::{c_int};
use std::ptr::{null,mut_null};
use swscale;
use util;
use video_decoder::VideoData;
use component_manager::Component;

pub struct VideoRenderer {
    component_id: int,
    width: int,
    height: int,
    pix_fmt: avutil::Enum_AVPixelFormat,
}

impl VideoRenderer {
    pub fn new(width: int, height: int, pix_fmt: avutil::Enum_AVPixelFormat)
            -> VideoRenderer {
        debug!("pix_fmt = {}", pix_fmt as int);
        VideoRenderer {
            component_id: -1,
            width: width,
            height: height,
            pix_fmt: pix_fmt,
        }
    }
    pub fn start(&self, vr_port: Port<Option<~VideoData>>) {
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
            while VideoRenderer::render(screen, frame_rgb.clone(),
                                        width, height,
                                        sws_ctx.clone(), &vr_port) {
                ;
            }
        }
    }

    fn render(screen: &sdl::video::Surface,
              frame_rgb: *mut avcodec::AVFrame,
              width: int, height: int,
              sws_ctx: *mut swscale::Struct_SwsContext,
              vr_port: &Port<Option<~VideoData>>) -> bool {
        match vr_port.recv() {
            Some(ref mut picture) => {
                let frame = picture.frame;
                let video_pts = picture.pts;
                screen.with_lock(|pixels| {
                    let ptr = pixels.as_mut_ptr();
                    unsafe {
                        avcodec::avpicture_fill(transmute(frame_rgb),
                                                transmute(ptr), avutil::AV_PIX_FMT_BGR24,
                                                width as c_int, height as c_int);
                        swscale::sws_scale(sws_ctx, transmute((*frame).data.as_ptr()),
                                           (*frame).linesize.as_ptr(), 0, (*frame).height,
                                           transmute((*frame_rgb).data.as_mut_ptr()),
                                           (*frame_rgb).linesize.as_ptr());
                    }
                });
                screen.flip();
                unsafe {
                    avcodec::avcodec_free_frame(transmute(&frame));
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

impl Component for VideoRenderer {
    fn set_id(&mut self, id: int) {
        self.component_id = id;
    }
    fn get_id(&self) -> int {
        self.component_id
    }
    fn get_name(&self) -> &str {
        "VideoRenderer"
    }
}
