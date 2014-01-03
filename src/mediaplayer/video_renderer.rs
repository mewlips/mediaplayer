use avcodec;
use avutil;
use sdl;
use std::cast::{transmute};
use std::libc::{c_int};
use std::ptr::{null,mut_null};
use swscale;
use component_manager::{Component,ComponentStruct,VideoRendererComponent,
                        ManagerComponent,Message,MsgStart,MsgVideoData};

pub struct VideoRenderer {
    component: Option<ComponentStruct>,
    width: int,
    height: int,
    pix_fmt: avutil::Enum_AVPixelFormat,
}

impl VideoRenderer {
    pub fn new(width: int, height: int, pix_fmt: avutil::Enum_AVPixelFormat)
            -> VideoRenderer {
        debug!("pix_fmt = {}", pix_fmt as int);
        VideoRenderer {
            component: Some(ComponentStruct::new(VideoRendererComponent)),
            width: width,
            height: height,
            pix_fmt: pix_fmt,
        }
    }
    pub fn start(&mut self) {
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
        let component = self.component.take().unwrap();
        do spawn {
            match component.recv() {
                Message { from: ManagerComponent, msg: MsgStart, .. } => {
                    info!("start VideoReneder");
                }
                _ => {
                    fail!("unexpected message received");
                }
            }
            while VideoRenderer::render(&component, screen, frame_rgb.clone(),
                                        width, height, sws_ctx.clone()) {
                ;
            }
        }
    }

    fn render(component: &ComponentStruct,
              screen: &sdl::video::Surface,
              frame_rgb: *mut avcodec::AVFrame,
              width: int, height: int,
              sws_ctx: *mut swscale::Struct_SwsContext) -> bool {
        match component.recv() {
            Message { msg: MsgVideoData(ref mut picture), .. } => {
                let frame = picture.frame;
                //let video_pts = picture.pts;
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
            _ => {
                // TODO
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
    fn get<'a>(&'a mut self) -> &'a mut ComponentStruct {
        self.component.get_mut_ref()
    }
}
