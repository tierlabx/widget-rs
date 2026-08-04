use crate::engine::{PetEngine, HEIGHT, WIDTH};
use gpui::*;
use image::{Frame, RgbaImage};
use smallvec::SmallVec;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

pub struct PetWidget {
    image_source: Option<ImageSource>,
    engine_rx: Option<Receiver<Vec<u8>>>,
}

impl PetWidget {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        println!("[PetWidget] new instance created!");
        let widget = Self {
            image_source: None,
            engine_rx: None,
        };
        widget.start_receiving(cx);
        widget
    }

    fn start_receiving(&self, cx: &mut Context<Self>) {
        println!("[PetWidget] start_receiving called!");
        let timer_executor = cx.background_executor().clone();
        let foreground = cx.foreground_executor().clone();
        let widget = cx.weak_entity();
        let mut async_cx = cx.to_async();

        foreground
            .spawn(async move {
                loop {
                    timer_executor.timer(Duration::from_millis(16)).await;

                    let is_enabled = async_cx
                        .update(|cx| {
                            cx.try_global::<widget_core::UIState>()
                                .map(|s| s.is_plugin_enabled("pet_plugin"))
                                .unwrap_or(true)
                        })
                        .unwrap_or(false);

                    let _ = widget.update(&mut async_cx, |this, cx| {
                        if !is_enabled {
                            if this.engine_rx.is_some() {
                                this.engine_rx = None;
                                this.image_source = None;
                                cx.notify();
                            }
                        } else {
                            if this.engine_rx.is_none() {
                                let (tx, rx) = channel();
                                PetEngine::start(tx);
                                this.engine_rx = Some(rx);
                            }

                            if let Some(rx) = &this.engine_rx {
                                let mut latest_frame = None;
                                while let Ok(frame) = rx.try_recv() {
                                    latest_frame = Some(frame);
                                }

                                if let Some(mut data) = latest_frame {
                                    for pixel in data.chunks_exact_mut(4) {
                                        pixel.swap(0, 2);
                                    }

                                    if let Some(rgba_image) =
                                        RgbaImage::from_raw(WIDTH, HEIGHT, data)
                                    {
                                        let frame = Frame::new(rgba_image);
                                        let render_image =
                                            RenderImage::new(SmallVec::from_elem(frame, 1));

                                        this.image_source =
                                            Some(ImageSource::Render(Arc::new(render_image)));
                                        cx.notify();
                                    }
                                }
                            }
                        }
                    });
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
                .bg(gpui::rgb(0xff0000))
                .child("Wait for pet...")
        }
    }
}
