use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, Textarea};
use gpui_component::{Icon, IconName};

use crate::model::STICKY_THEMES;
use crate::view::StickyWidget;

/// 渲染便签右侧便签纸主内容区（标题栏、调色板、图片预览、多行编辑区）
pub(crate) fn render_content_panel(
    view: &StickyWidget,
    cx: &mut Context<StickyWidget>,
) -> impl IntoElement {
    let current_color_index = view.data.current().color_index.min(STICKY_THEMES.len() - 1);
    let theme = &STICKY_THEMES[current_color_index];
    let (bg_color, header_color, text_color, border_color) = (
        rgb(theme.bg_hex),
        rgb(theme.header_hex),
        rgb(theme.text_hex),
        rgb(theme.border_hex),
    );
    let show_palette = view.show_palette;
    let images = &view.data.current().images;

    // 半透明文字色
    let h = ((theme.text_hex >> 16) & 0xFF) as f32 / 255.0;
    let s = ((theme.text_hex >> 8) & 0xFF) as f32 / 255.0;
    let l = (theme.text_hex & 0xFF) as f32 / 255.0;
    let text_muted = hsla(h / 360.0, s, l, 0.55);
    let text_faint = hsla(h / 360.0, s, l, 0.28);

    div()
        .flex_1()
        .h_full()
        .flex()
        .flex_col()
        .bg(bg_color)
        .border_1()
        .border_color(border_color)
        .overflow_hidden()
        .min_h_0()
        // ── Header ────────────────────────────────────────────
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .px(px(8.0))
                .py(px(4.0))
                .bg(header_color)
                .border_b_1()
                .border_color(border_color)
                .flex_shrink_0()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_muted)
                        .child(
                            Input::new(&view.title_input)
                                .appearance(false)
                                .bordered(false),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(2.0))
                        .flex_shrink_0()
                        .child(
                            div()
                                .w(px(22.0))
                                .h(px(22.0))
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
                                .child(Icon::new(IconName::Palette).size(px(12.0))),
                        )
                        .child(
                            div()
                                .w(px(22.0))
                                .h(px(22.0))
                                .flex()
                                .justify_center()
                                .items_center()
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .text_color(text_faint)
                                .hover(|s| s.bg(rgba(0xff000018)).text_color(rgb(0xff4444)))
                                .id("sticky-del")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.data.delete_current();
                                    this.sync_inputs_from_current(window, cx);
                                    this.save_now(cx);
                                    cx.notify();
                                }))
                                .child(Icon::new(IconName::Delete).size(px(12.0))),
                        ),
                ),
        )
        // ── 调色板 ────────────────────────────────────────────
        .when(show_palette, |d| {
            d.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .bg(header_color)
                    .border_b_1()
                    .border_color(border_color)
                    .flex_shrink_0()
                    .children(STICKY_THEMES.iter().enumerate().map(|(i, t)| {
                        let is_active = i == current_color_index;
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
                                this.save_now(cx);
                                cx.notify();
                            }))
                    })),
            )
        })
        // ── 图片预览区 ────────────────────────────────────────
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
                            .w(px(60.0))
                            .h(px(60.0))
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
                                    .id(ElementId::Name(format!("sticky-del-img-{img_idx}").into()))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let imgs = &mut this.data.current_mut().images;
                                        if img_idx < imgs.len() {
                                            imgs.remove(img_idx);
                                        }
                                        this.save_now(cx);
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Close).size(px(7.0))),
                            )
                    })),
            )
        })
        // ── 文本编辑区 ────────────────────────────────────────
        .child(
            div()
                .flex_1()
                .w_full()
                .min_h_0()
                .overflow_hidden()
                .text_sm()
                .text_color(text_color)
                .child(
                    Textarea::new(&view.input)
                        .appearance(false)
                        .bordered(false)
                        .size_full(),
                ),
        )
}
