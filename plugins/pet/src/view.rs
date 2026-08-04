use crate::engine::{PetEngine, HEIGHT, WIDTH};
use gpui::*;
use image::{Frame, RgbaImage};
use smallvec::SmallVec;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

pub struct PetWidget {
    image_source: Option<ImageSource>,
}

impl PetWidget {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (tx, rx) = channel();

        PetEngine::start(tx);

        let widget = Self { image_source: None };
        widget.start_receiving(rx, cx);
        widget
    }

    fn start_receiving(&self, rx: Receiver<Vec<u8>>, cx: &mut Context<Self>) {
        let timer_executor = cx.background_executor().clone();
        let foreground = cx.foreground_executor().clone();
        let widget = cx.weak_entity();
        let mut async_cx = cx.to_async();

        foreground
            .spawn(async move {
                loop {
                    timer_executor.timer(Duration::from_millis(16)).await;

                    let mut latest_frame = None;
                    while let Ok(frame) = rx.try_recv() {
                        latest_frame = Some(frame);
                    }

                    if let Some(mut data) = latest_frame {
                        // Swap RGBA to BGRA for GPUI
                        for pixel in data.chunks_exact_mut(4) {
                            pixel.swap(0, 2);
                        }

                        if let Some(rgba_image) = RgbaImage::from_raw(WIDTH, HEIGHT, data) {
                            let frame = Frame::new(rgba_image);
                            let render_image = RenderImage::new(SmallVec::from_elem(frame, 1));

                            let _ = widget.update(&mut async_cx, |this, cx| {
                                this.image_source =
                                    Some(ImageSource::Render(Arc::new(render_image)));
                                cx.notify();
                            });
                        }
                    }
                }
            })
            .detach();
    }
}

impl Render for PetWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(src) = &self.image_source {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    img(src.clone())
                        .w_full()
                        .h_full()
                        .object_fit(ObjectFit::Contain),
                )
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child("Loading Pet...")
        }
    }
}
