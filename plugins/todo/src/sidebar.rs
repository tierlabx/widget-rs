use gpui::*;
use gpui_component::{Icon, IconName};

use crate::model::{TodoTag, GANTT_COLORS};

/// 渲染左侧吸附式分类标签侧边栏
pub fn render_sidebar<V: 'static>(
    tags: &[TodoTag],
    active_tag_id: &str,
    on_select_tag: impl Fn(&mut V, &mut Window, &mut Context<V>, String) + 'static + Clone,
    on_edit_tag: impl Fn(&mut V, &mut Window, &mut Context<V>, TodoTag) + 'static + Clone,
    on_add_tag_click: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let active_tag_id = active_tag_id.to_string();

    div()
        .w(px(46.0))
        .flex()
        .flex_col()
        .gap(px(3.0))
        .pt(px(16.0))
        .pb(px(8.0))
        .overflow_hidden()
        // 1. "全部" 分类 Tab
        .child({
            let is_active = active_tag_id == "all";
            let on_select = on_select_tag.clone();
            div()
                .relative()
                .w_full()
                .h(px(32.0))
                .rounded_l(px(8.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_xs()
                .font_weight(if is_active {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if is_active {
                    rgb(0x0f172a)
                } else {
                    rgba(0xffffffcc)
                })
                .bg(if is_active {
                    rgb(0x38bdf8)
                } else {
                    rgba(0x0f172a65)
                })
                .border_1()
                .border_color(if is_active {
                    rgb(0xbae6fd)
                } else {
                    rgba(0xffffff15)
                })
                .hover(|s| {
                    s.bg(if is_active {
                        rgb(0x38bdf8)
                    } else {
                        rgba(0x1e293ba0)
                    })
                })
                .id("todo-tab-all")
                .on_click(cx.listener(move |this, _, window, cx| {
                    on_select(this, window, cx, "all".to_string());
                }))
                .child("全部")
        })
        // 2. 各自定义分类 Tab
        .children(tags.iter().map(|tag| {
            let is_active = active_tag_id == tag.id;
            let tag_clone = tag.clone();
            let tag_id_clone = tag.id.clone();
            let tag_color = &GANTT_COLORS[tag.gantt_color % GANTT_COLORS.len()];
            let on_select = on_select_tag.clone();
            let on_edit = on_edit_tag.clone();
            let contrast_text = tag_color.contrast_text();

            div()
                .relative()
                .w_full()
                .h(px(32.0))
                .rounded_l(px(8.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_xs()
                .font_weight(if is_active {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(if is_active {
                    contrast_text
                } else {
                    rgba(0xffffffcc).into()
                })
                .bg(if is_active {
                    rgb(tag_color.hex)
                } else {
                    rgba(0x0f172a65)
                })
                .border_1()
                .border_color(if is_active {
                    rgb(0xffffff)
                } else {
                    rgba(0xffffff15)
                })
                .hover(|s| {
                    s.bg(if is_active {
                        rgb(tag_color.hex)
                    } else {
                        rgba(0x1e293ba0)
                    })
                })
                .id(ElementId::Name(format!("todo-tab-{}", tag.id).into()))
                .on_click(cx.listener({
                    let tag_id = tag_id_clone.clone();
                    let on_select = on_select.clone();
                    move |this, event: &ClickEvent, window, cx| {
                        if event.click_count() >= 2 {
                            on_edit(this, window, cx, tag_clone.clone());
                        } else {
                            on_select(this, window, cx, tag_id.clone());
                        }
                    }
                }))
                .child(
                    div()
                        .absolute()
                        .left(px(3.0))
                        .w(px(4.0))
                        .h(px(4.0))
                        .rounded_full()
                        .bg(if is_active {
                            contrast_text
                        } else {
                            rgb(tag_color.hex).into()
                        }),
                )
                .child(tag.name.clone())
        }))
        // 3. 底部“+”新建分类按钮
        .child(
            div()
                .w_full()
                .h(px(28.0))
                .rounded_l(px(6.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_color(rgba(0xffffff50))
                .bg(rgba(0x00000030))
                .border_1()
                .border_color(rgba(0xffffff10))
                .hover(|s| s.bg(rgba(0x38bdf825)).text_color(rgb(0x38bdf8)))
                .id("todo-add-tag-btn")
                .on_click(cx.listener(move |this, _, window, cx| {
                    on_add_tag_click(this, window, cx);
                }))
                .child(Icon::new(IconName::Plus).size(px(12.0))),
        )
}
