use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Icon, IconName, InteractiveElementExt};
use raw_window_handle::HasWindowHandle;

use crate::model::StickyModel;
use gpui_component::ActiveTheme;

pub struct StickyWidget {
    hwnd_reported: bool,
    input: Entity<InputState>,
    is_preview: bool,
}

impl StickyWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 从全局配置读取已保存的便签内容
        let saved_content = StickyModel::load(cx);

        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .default_value(saved_content)
                .placeholder("在这里记录你的想法...")
        });

        // 内容变化时更新内存 + 立即写盘
        cx.subscribe(
            &input,
            |_this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                // Change 事件在每次文字变化时触发
                if let InputEvent::Change = event {
                    let text = input.read(cx).value().to_string();
                    StickyModel::save(&text, cx);
                }
            },
        )
        .detach();

        Self {
            hwnd_reported: false,
            input,
            is_preview: true,
        }
    }
}

impl Render for StickyWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit_mode = cx
            .try_global::<widget_core::UIState>()
            .is_some_and(|s| s.is_edit_mode);

        if let Ok(handle) = _window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
                let hwnd = h.hwnd.get();
                if !self.hwnd_reported {
                    self.hwnd_reported = true;
                    let _ = hwnd;
                }
            }
        }
        widget_core::update_window_edit_mode(_window, is_edit_mode);

        let drag_handle = if is_edit_mode {
            Some(
                div()
                    .w_full()
                    .h(px(28.0))
                    .bg(rgb(0x00d992)) // Emerald Signal Green
                    .flex()
                    .justify_center()
                    .items_center()
                    .id("sticky-drag")
                    .cursor_pointer()
                    .hover(|s| s.bg(rgba(0x00d992cc)))
                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                        widget_core::start_window_drag(window);
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x050507))
                            .child(":: 拖拽移动便签 ::"),
                    ),
            )
        } else {
            None
        };

        let input = &self.input;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgba(0x050507d9))
            .border_1()
            .border_color(if is_edit_mode {
                rgb(0x00d992)
            } else {
                rgb(0x3d3a39)
            })
            .children(drag_handle)
            // 标题栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .w_full()
                    .px(px(14.0))
                    .py(px(10.0))
                    .bg(rgb(0xfef3c7)) // 保持暖黄
                    .border_b_1()
                    .border_color(rgba(0xf59e0b80)) // 加深下划线
                    .child(
                        div()
                            .text_color(rgb(0x92400e))
                            .child(Icon::new(IconName::File).size(px(14.0))),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x78350f))
                            .child("便签"),
                    ),
            )
            // 文本区域
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .w_full()
                    .bg(rgb(0xfef3c7))
                    .p(px(8.0))
                    .id("sticky-container")
                    .on_double_click(|_, _, cx| {
                        cx.update_global::<widget_core::UIState, _>(|s, _| {
                            // 仅在非排版模式下，双击切换预览/编辑
                            if !s.is_edit_mode {
                                // 略
                            }
                        });
                    })
                    .on_mouse_down(MouseButton::Left, move |_, _, _| {
                        // 拦截双击事件
                    })
                    .child(
                        div()
                            .id("sticky-content")
                            .flex_1()
                            .w_full()
                            .h_full()
                            .bg(if self.is_preview {
                                gpui::Hsla::transparent_black()
                            } else {
                                cx.theme().background
                            })
                            .border_1()
                            .border_color(if self.is_preview {
                                rgba(0x00000000)
                            } else {
                                rgba(0xf59e0b20)
                            })
                            .rounded(px(4.0))
                            .p(px(8.0))
                            .text_color(rgb(0x3d2000))
                            .hover(|s| {
                                if !self.is_preview {
                                    s.border_color(rgba(0xf59e0b60))
                                } else {
                                    s
                                }
                            })
                            .on_double_click(cx.listener(|this, _, _, cx| {
                                this.is_preview = !this.is_preview;
                                cx.notify();
                            }))
                            .child(if self.is_preview {
                                crate::markdown::render_markdown(
                                    &input.read(cx).value().to_string(),
                                )
                                .into_any_element()
                            } else {
                                Input::new(input)
                                    .h_full()
                                    .appearance(false)
                                    .bordered(false)
                                    .into_any_element()
                            }),
                    ),
            )
    }
}
