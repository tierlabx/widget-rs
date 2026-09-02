use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Icon, IconName};

use crate::model::{StickyData, StickyModel, STICKY_THEMES};

pub struct StickyWidget {
    input: Entity<InputState>,
    data: StickyData,
    show_palette: bool,
    pending_input_reset: bool,
}

impl StickyWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = StickyModel::load(cx);
        let input = Self::make_input(window, cx, &data);
        Self {
            input,
            data,
            show_palette: false,
            pending_input_reset: false,
        }
    }

    fn make_input(
        window: &mut Window,
        cx: &mut Context<Self>,
        data: &StickyData,
    ) -> Entity<InputState> {
        let content = data.current().content.clone();
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .default_value(content)
                .placeholder("在这里记录你的想法...")
        });
        cx.subscribe(
            &input,
            |this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    this.data.current_mut().content = input.read(cx).value().to_string();
                    StickyModel::save(&this.data, cx);
                }
            },
        )
        .detach();
        input
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 确保在便签窗口中，Theme 文本、光标与选区颜色为深炭黑墨水色（防止深色主题下的白色光标在浅色便签上隐形）
        let theme_mut = gpui_component::Theme::global_mut(cx);
        theme_mut.colors.foreground = gpui::hsla(0.0, 0.0, 0.12, 1.0);
        theme_mut.colors.muted_foreground = gpui::hsla(0.0, 0.0, 0.40, 1.0);
        theme_mut.colors.caret = gpui::hsla(0.0, 0.0, 0.10, 1.0);
        theme_mut.colors.selection = gpui::hsla(0.0, 0.0, 0.0, 0.15);

        if self.pending_input_reset {
            self.pending_input_reset = false;
            self.input = Self::make_input(window, cx, &self.data);
        }

        let current_note = self.data.current().clone();
        let theme = &STICKY_THEMES[current_note.color_index.min(STICKY_THEMES.len() - 1)];
        let bg_color = rgb(theme.bg_hex);
        let header_color = rgb(theme.header_hex);
        let text_color = rgb(theme.text_hex);
        let border_color = rgb(theme.border_hex);

        let total = self.data.notes.len();
        let current_idx = self.data.current_index;
        let show_palette = self.show_palette;
        let has_prev = current_idx > 0;
        let has_next = current_idx + 1 < total;
        let new_input = self.input.clone();
        let images = current_note.images.clone();

        // 用 hsla 构造半透明文字色（text_hex 的 50% / 20% 透明版本）
        let h = ((theme.text_hex >> 16) & 0xFF) as f32 / 255.0;
        let s = ((theme.text_hex >> 8) & 0xFF) as f32 / 255.0;
        let l = (theme.text_hex & 0xFF) as f32 / 255.0;
        let text_muted = hsla(h / 360.0, s, l, 0.55);
        let text_faint = hsla(h / 360.0, s, l, 0.28);

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .min_h_0()
            .on_drop(cx.listener(|this, paths: &gpui::ExternalPaths, _, cx| {
                for path in paths.paths() {
                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if ["png", "jpg", "jpeg", "webp", "gif", "bmp"].contains(&ext.as_str()) {
                        this.data
                            .current_mut()
                            .images
                            .push(path.to_string_lossy().to_string());
                        StickyModel::save(&this.data, cx);
                        cx.notify();
                    }
                }
            }))
            // ── Header ─────────────────────────────────────────────────
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px(px(8.0))
                    .py(px(5.0))
                    .bg(header_color)
                    .border_b_1()
                    .border_color(border_color)
                    .flex_shrink_0()
                    // 左：翻页导航
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(if has_prev { text_muted } else { text_faint })
                                    .hover(|s| s.bg(rgba(0x00000018)))
                                    .id("sticky-prev")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if has_prev {
                                            this.data.prev();
                                            this.pending_input_reset = true;
                                            StickyModel::save(&this.data, cx);
                                            cx.notify();
                                        }
                                    }))
                                    .child(Icon::new(IconName::ChevronLeft).size(px(13.0))),
                            )
                            .child(div().text_xs().text_color(text_faint).child(format!(
                                "{}/{}",
                                current_idx + 1,
                                total
                            )))
                            .child(
                                div()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(if has_next { text_muted } else { text_faint })
                                    .hover(|s| s.bg(rgba(0x00000018)))
                                    .id("sticky-next")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if has_next {
                                            this.data.next();
                                            this.pending_input_reset = true;
                                            StickyModel::save(&this.data, cx);
                                            cx.notify();
                                        }
                                    }))
                                    .child(Icon::new(IconName::ChevronRight).size(px(13.0))),
                            ),
                    )
                    // 右：操作按钮
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            // 颜色
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(text_faint)
                                    .hover(|s| s.bg(rgba(0x00000018)).text_color(text_color))
                                    .id("sticky-palette")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_palette = !this.show_palette;
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Palette).size(px(13.0))),
                            )
                            // 删除
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(text_faint)
                                    .hover(|s| s.bg(rgba(0xff000018)).text_color(rgb(0xff4444)))
                                    .id("sticky-del")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.data.delete_current();
                                        this.pending_input_reset = true;
                                        StickyModel::save(&this.data, cx);
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Delete).size(px(13.0))),
                            )
                            // 新建
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(text_muted)
                                    .hover(|s| s.bg(rgba(0x00000018)).text_color(text_color))
                                    .id("sticky-new")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.data.new_note();
                                        this.pending_input_reset = true;
                                        StickyModel::save(&this.data, cx);
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Plus).size(px(14.0))),
                            ),
                    ),
            )
            // ── 颜色调色板 ─────────────────────────────────────────────
            .when(show_palette, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(10.0))
                        .py(px(6.0))
                        .bg(header_color)
                        .border_b_1()
                        .border_color(border_color)
                        .flex_shrink_0()
                        .children(STICKY_THEMES.iter().enumerate().map(|(i, t)| {
                            let is_active = i == current_note.color_index;
                            div()
                                .w(px(18.0))
                                .h(px(18.0))
                                .rounded_full()
                                .cursor_pointer()
                                .bg(rgb(t.bg_hex))
                                .border_2()
                                .border_color(if is_active {
                                    rgb(0x444444)
                                } else {
                                    rgb(t.border_hex)
                                })
                                .id(ElementId::Name(format!("sticky-color-{i}").into()))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.data.current_mut().color_index = i;
                                    this.show_palette = false;
                                    StickyModel::save(&this.data, cx);
                                    cx.notify();
                                }))
                        })),
                )
            })
            // ── 图片预览区 ─────────────────────────────────────────────
            .when(!images.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(4.0))
                        .p(px(6.0))
                        .border_b_1()
                        .border_color(border_color)
                        .flex_shrink_0()
                        .children(images.iter().enumerate().map(|(img_idx, path)| {
                            div()
                                .relative()
                                .w(px(64.0))
                                .h(px(64.0))
                                .rounded(px(4.0))
                                .overflow_hidden()
                                .border_1()
                                .border_color(border_color)
                                .child(
                                    img(std::path::PathBuf::from(path))
                                        .w_full()
                                        .h_full()
                                        .object_fit(ObjectFit::Cover),
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .top(px(2.0))
                                        .right(px(2.0))
                                        .w(px(14.0))
                                        .h(px(14.0))
                                        .rounded_full()
                                        .bg(rgba(0x00000088))
                                        .text_color(rgb(0xffffff))
                                        .flex()
                                        .justify_center()
                                        .items_center()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgba(0xff000088)))
                                        .id(ElementId::Name(
                                            format!("sticky-del-img-{img_idx}").into(),
                                        ))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let imgs = &mut this.data.current_mut().images;
                                            if img_idx < imgs.len() {
                                                imgs.remove(img_idx);
                                            }
                                            StickyModel::save(&this.data, cx);
                                            cx.notify();
                                        }))
                                        .child(Icon::new(IconName::Close).size(px(7.0))),
                                )
                        })),
                )
            })
            // ── 文本编辑区 ─────────────────────────────────────────────
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h_0()
                    .overflow_hidden()
                    .text_sm()
                    .text_color(text_color)
                    .child(
                        Input::new(&new_input)
                            .appearance(false)
                            .bordered(false)
                            .size_full(),
                    ),
            )
    }
}
