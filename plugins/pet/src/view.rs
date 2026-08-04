use crate::engine::{EngineCommand, PetEngine, HEIGHT, WIDTH};
use gpui::*;
use image::{Frame, RgbaImage};
use smallvec::SmallVec;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

pub struct PetWidget {
    image_source: Option<ImageSource>,
    engine_rx: Option<Receiver<Vec<u8>>>,
    engine_cmd_tx: Option<Sender<EngineCommand>>,
    show_menu: bool,
    last_window_pos: Option<Point<gpui::Pixels>>,
}

impl PetWidget {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        println!("[PetWidget] new instance created!");
        let widget = Self {
            image_source: None,
            engine_rx: None,
            engine_cmd_tx: None,
            show_menu: false,
            last_window_pos: None,
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
                                let config = cx
                                    .try_global::<widget_core::AppConfig>()
                                    .and_then(|c| {
                                        c.get_plugin_data::<crate::PetConfig>("pet_plugin")
                                    })
                                    .unwrap_or_default();

                                let (tx, rx) = sync_channel(2);
                                let (cmd_tx, cmd_rx) = channel();
                                PetEngine::start(tx, cmd_rx, config.model_path, config.fps);
                                this.engine_rx = Some(rx);
                                this.engine_cmd_tx = Some(cmd_tx);
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bounds = match _window.window_bounds() {
            WindowBounds::Fullscreen(b)
            | WindowBounds::Maximized(b)
            | WindowBounds::Windowed(b) => b,
        };
        let current_pos = bounds.origin;

        if let Some(last_pos) = self.last_window_pos {
            let dx: f32 = (current_pos.x - last_pos.x).into();
            let dy: f32 = (current_pos.y - last_pos.y).into();

            if dx.abs() > 0.1 || dy.abs() > 0.1 {
                if let Some(tx) = &self.engine_cmd_tx {
                    let _ = tx.send(EngineCommand::UpdateVelocity(dx, dy));
                }
            } else {
                if let Some(tx) = &self.engine_cmd_tx {
                    let _ = tx.send(EngineCommand::UpdateVelocity(0.0, 0.0));
                }
            }
        }
        self.last_window_pos = Some(current_pos);

        let pet_img = if let Some(src) = &self.image_source {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &gpui::MouseDownEvent, _win, _cx| {
                        widget_core::start_window_drag(_win);
                        if let Some(tx) = &this.engine_cmd_tx {
                            let event_x: f32 = event.position.x.into();
                            let event_y: f32 = event.position.y.into();
                            let _ = tx.send(EngineCommand::Touch(event_x, event_y));
                        }
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(|this, _, _win, cx| {
                        this.show_menu = !this.show_menu;
                        cx.notify();
                    }),
                )
                .child(
                    img(src.clone())
                        .w_full()
                        .h_full()
                        .object_fit(ObjectFit::Contain),
                )
        } else {
            div().size_full()
        };

        let menu = if self.show_menu {
            div()
                .absolute()
                .top(px(10.0))
                .left(px(10.0))
                .p(px(8.0))
                .rounded(px(8.0))
                .bg(rgba(0x000000d0))
                .border_1()
                .border_color(rgba(0xffffff40))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .id("load_vrm_btn")
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgba(0xffffff20)))
                        .p(px(6.0))
                        .rounded(px(4.0))
                        .on_click(cx.listener(|this, _, _win, cx| {
                            this.show_menu = false;

                            let foreground = cx.foreground_executor().clone();
                            let cmd_tx = this.engine_cmd_tx.clone();
                            let async_cx = cx.to_async();
                            foreground
                                .spawn(async move {
                                    let path = rfd::AsyncFileDialog::new()
                                        .add_filter("VRM", &["vrm"])
                                        .pick_file()
                                        .await;

                                    if let Some(file) = path {
                                        let path_str = file.path().display().to_string();
                                        if let Some(tx) = cmd_tx {
                                            let _ = tx.send(EngineCommand::LoadModel(
                                                path_str.to_string(),
                                            ));
                                        }
                                        let _ = async_cx.update(|cx| {
                                            cx.update_global::<widget_core::AppConfig, _>(
                                                |c, _cx| {
                                                    let mut config = c
                                                        .get_plugin_data::<crate::PetConfig>(
                                                            "pet_plugin",
                                                        )
                                                        .unwrap_or_default();
                                                    config.model_path = path_str.clone();
                                                    c.set_plugin_data("pet_plugin", &config);
                                                },
                                            );
                                            widget_core::save_config_now(cx);
                                        });
                                    }
                                })
                                .detach();
                        }))
                        .child("📂 加载自定义 VRM 模型..."),
                )
                .child(
                    div()
                        .id("fps_60_btn")
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgba(0xffffff20)))
                        .p(px(6.0))
                        .rounded(px(4.0))
                        .on_click(cx.listener(|this, _, _win, cx| {
                            let fps = 60;
                            if let Some(tx) = &this.engine_cmd_tx {
                                let _ = tx.send(EngineCommand::SetFps(fps));
                            }
                            cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                let mut config = c
                                    .get_plugin_data::<crate::PetConfig>("pet_plugin")
                                    .unwrap_or_default();
                                config.fps = fps;
                                c.set_plugin_data("pet_plugin", &config);
                            });
                            widget_core::save_config_now(cx);
                            this.show_menu = false;
                        }))
                        .child("⚡ 设为 60 FPS"),
                )
                .child(
                    div()
                        .id("fps_30_btn")
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgba(0xffffff20)))
                        .p(px(6.0))
                        .rounded(px(4.0))
                        .on_click(cx.listener(|this, _, _win, cx| {
                            let fps = 30;
                            if let Some(tx) = &this.engine_cmd_tx {
                                let _ = tx.send(EngineCommand::SetFps(fps));
                            }
                            cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                let mut config = c
                                    .get_plugin_data::<crate::PetConfig>("pet_plugin")
                                    .unwrap_or_default();
                                config.fps = fps;
                                c.set_plugin_data("pet_plugin", &config);
                            });
                            widget_core::save_config_now(cx);
                            this.show_menu = false;
                        }))
                        .child("🐢 设为 30 FPS (省电)"),
                )
        } else {
            div()
        };

        div()
            .id("pet_container")
            .size_full()
            .relative()
            .child(pet_img)
            .child(menu)
    }
}
