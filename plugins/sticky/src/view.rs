use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{ActiveTheme, Icon, IconName, InteractiveElementExt};

use crate::model::StickyModel;

pub struct StickyWidget {
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
            input,
            is_preview: true,
        }
    }
}

impl widget_core::WidgetContent for StickyWidget {
    fn plugin_id(&self) -> &'static str {
        "sticky_widget"
    }

    fn drag_label(&self) -> &'static str {
        "拖拽移动便签"
    }
}

impl Render for StickyWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = &self.input;

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .bg(rgba(0x050507d9))
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
                    .overflow_hidden()
                    .min_h_0()
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
                            .overflow_hidden()
                            .min_h_0()
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
