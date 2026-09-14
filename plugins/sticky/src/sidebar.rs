use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{Icon, IconName};

use crate::model::{StickyNote, STICKY_THEMES};

/// 便签拖拽排序载荷
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraggedNote {
    pub from_idx: usize,
}

/// 便签拖拽跟随预览
struct NoteDragPreview {
    text: String,
    bg_hex: u32,
    text_hex: u32,
}

impl Render for NoteDragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.0))
            .py(px(5.0))
            .bg(rgb(self.bg_hex))
            .rounded(px(6.0))
            .border_1()
            .border_color(rgb(self.text_hex))
            .shadow_lg()
            .text_xs()
            .text_color(rgb(self.text_hex))
            .child(self.text.clone())
    }
}

/// 渲染便签左侧紧凑 Tab 侧边栏（宽度 42px，支持拖拽排序）
pub fn render_sidebar<V: 'static>(
    notes: &[StickyNote],
    current_idx: usize,
    on_select: impl Fn(&mut V, &mut Window, &mut Context<V>, usize) + 'static + Clone,
    on_new: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_reorder: impl Fn(&mut V, &mut Window, &mut Context<V>, usize, usize) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let note_count = notes.len();

    div()
        .w(px(42.0))
        .flex()
        .flex_col()
        .items_stretch()
        .gap(px(3.0))
        .pt(px(10.0))
        .pb(px(6.0))
        .overflow_hidden()
        // 各便签 Tab
        .children(notes.iter().enumerate().map(|(idx, note)| {
            let is_active = idx == current_idx;
            let note_theme = &STICKY_THEMES[note.color_index.min(STICKY_THEMES.len() - 1)];
            let tab_label = note.tab_label(idx);

            let active_bg = rgb(note_theme.header_hex);
            let active_border = rgb(note_theme.border_hex);
            let active_text: Hsla = if note_theme.is_dark {
                rgb(0xf0f0f0).into()
            } else {
                rgb(note_theme.text_hex).into()
            };

            let preview_text = note.tab_label(idx);
            let preview_bg = note_theme.bg_hex;
            let preview_text_hex = note_theme.text_hex;
            let on_reorder = on_reorder.clone();
            let on_select = on_select.clone();

            div()
                .relative()
                .w_full()
                .h(px(32.0))
                .rounded_l(px(7.0))
                .px(px(4.0))
                .flex()
                .items_center()
                .justify_center()
                .overflow_hidden()
                .cursor_grab()
                .text_size(px(10.5))
                .font_weight(if is_active {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(if is_active {
                    active_text
                } else {
                    rgba(0x000000bb).into()
                })
                .bg(if is_active {
                    active_bg
                } else {
                    rgba(0xffffff35)
                })
                .border_1()
                .border_color(if is_active {
                    active_border
                } else {
                    rgba(0x00000015)
                })
                .hover(|s| {
                    s.bg(if is_active {
                        active_bg
                    } else {
                        rgba(0xffffff60)
                    })
                })
                .id(ElementId::Name(format!("sticky-tab-{idx}").into()))
                .drag_over::<DraggedNote>(|s, _, _, _| {
                    s.border_color(rgb(note_theme.border_hex))
                        .bg(rgba(note_theme.bg_hex | 0x99))
                })
                .on_drop(cx.listener(move |this, drag: &DraggedNote, window, cx| {
                    on_reorder(this, window, cx, drag.from_idx, idx);
                }))
                .on_drag(DraggedNote { from_idx: idx }, move |_, _, _, cx| {
                    cx.new(|_| NoteDragPreview {
                        text: preview_text.clone(),
                        bg_hex: preview_bg,
                        text_hex: preview_text_hex,
                    })
                })
                .on_click(cx.listener(move |this, _, window, cx| {
                    on_select(this, window, cx, idx);
                }))
                // 激活时左侧竖条指示
                .when(is_active, |d: Stateful<Div>| {
                    d.child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(6.0))
                            .bottom(px(6.0))
                            .w(px(3.0))
                            .rounded_r(px(2.0))
                            .bg(rgb(note_theme.text_hex)),
                    )
                })
                .child(
                    div()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_align(TextAlign::Center)
                        .child(tab_label),
                )
        }))
        // 底部新建便签按钮
        .child({
            let on_new = on_new.clone();
            div()
                .w_full()
                .h(px(26.0))
                .rounded_l(px(6.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_color(rgba(0x00000060))
                .bg(rgba(0xffffff20))
                .border_1()
                .border_color(rgba(0x00000015))
                .hover(|s| s.bg(rgba(0xffffff60)).text_color(rgba(0x000000cc)))
                .id("sticky-new-tab")
                .on_click(cx.listener(move |this, _, window, cx| {
                    on_new(this, window, cx);
                }))
                .child(Icon::new(IconName::Plus).size(px(11.0)))
        })
        // 便签数量指示
        .when(note_count > 1, |d: Div| {
            d.child(
                div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .text_size(px(9.0))
                    .text_color(rgba(0x00000055))
                    .child(format!("{}/{}", current_idx + 1, note_count)),
            )
        })
}
