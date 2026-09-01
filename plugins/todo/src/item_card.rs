use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};

use crate::item_detail::render_item_detail;
use crate::model::{ReminderRule, TodoItem, TodoTag, GANTT_COLORS};

/// 单条待办项的渲染参数
pub struct ItemCardProps<'a> {
    pub idx: usize,
    pub item: &'a TodoItem,
    pub tags: &'a [TodoTag],
    pub active_tag_id: &'a str,
    pub is_editing: bool,
    pub is_expanded: bool,
    pub edit_input: &'a Entity<InputState>,
}

use std::rc::Rc;

/// 待办项交互回调集合
#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct ItemCardCallbacks<V: 'static> {
    pub on_toggle_done: Rc<dyn Fn(&mut V, &mut Window, &mut Context<V>, usize)>,
    pub on_toggle_expand: Rc<dyn Fn(&mut V, &mut Window, &mut Context<V>, usize)>,
    pub on_start_edit: Rc<dyn Fn(&mut V, &mut Window, &mut Context<V>, usize)>,
    pub on_confirm_edit: Rc<dyn Fn(&mut V, &mut Window, &mut Context<V>, usize)>,
    pub on_delete_item: Rc<dyn Fn(&mut V, &mut Window, &mut Context<V>, usize)>,
    pub on_change_tag: Rc<dyn Fn(&mut V, &mut Window, &mut Context<V>, usize, String)>,
    pub on_set_reminder:
        Rc<dyn Fn(&mut V, &mut Window, &mut Context<V>, usize, Option<ReminderRule>)>,
    pub on_set_color: Rc<dyn Fn(&mut V, &mut Window, &mut Context<V>, usize, usize)>,
}

/// 渲染单条待办项（包括编辑态与正常展示态）
pub fn render_todo_item<V: 'static>(
    props: ItemCardProps,
    callbacks: &ItemCardCallbacks<V>,
    cx: &mut Context<V>,
) -> AnyElement {
    let idx = props.idx;
    let done = props.item.done;
    let text = props.item.text.clone();
    let is_editing = props.is_editing;
    let is_expanded = props.is_expanded;
    let color_idx = props.item.gantt_color % GANTT_COLORS.len();
    let gantt = &GANTT_COLORS[color_idx];
    let item_tag = props
        .tags
        .iter()
        .find(|t| t.id == props.item.tag_id)
        .cloned();
    let reminder_text = props.item.reminder.as_ref().map(|r| r.display_text());
    let active_tag_id = props.active_tag_id.to_string();
    let all_tags = props.tags.to_vec();

    if is_editing {
        let on_confirm = callbacks.on_confirm_edit.clone();
        let on_delete = callbacks.on_delete_item.clone();
        div()
            .flex()
            .items_center()
            .w_full()
            .px(px(8.0))
            .py(px(6.0))
            .gap(px(6.0))
            .bg(rgba(0x0f172af0))
            .rounded(px(8.0))
            .border_1()
            .border_color(rgb(0x38bdf8))
            .child(
                div().flex_1().child(
                    Input::new(props.edit_input)
                        .appearance(false)
                        .bordered(false),
                ),
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
                    .bg(rgba(0x00d99220))
                    .text_color(rgb(0x00d992))
                    .hover(|s| s.bg(rgba(0x00d99245)))
                    .id(ElementId::Name(format!("todo-confirm-{idx}").into()))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        on_confirm(this, window, cx, idx);
                    }))
                    .child(Icon::new(IconName::Check).size(px(11.0))),
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
                    .bg(rgba(0xff4d4d20))
                    .text_color(rgb(0xff6b6b))
                    .hover(|s| s.bg(rgba(0xff4d4d40)))
                    .id(ElementId::Name(format!("todo-del-{idx}").into()))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        on_delete(this, window, cx, idx);
                    }))
                    .child(Icon::new(IconName::Delete).size(px(11.0))),
            )
            .into_any_element()
    } else {
        let on_toggle = callbacks.on_toggle_done.clone();
        let on_expand = callbacks.on_toggle_expand.clone();
        let on_edit = callbacks.on_start_edit.clone();
        let on_delete = callbacks.on_delete_item.clone();

        div()
            .id(ElementId::Name(format!("todo-item-{idx}").into()))
            .flex()
            .flex_col()
            .w_full()
            .bg(if done {
                rgba(0x0f172a65)
            } else {
                rgba(0x0f172ad0)
            })
            .rounded(px(8.0))
            .border_1()
            .border_color(if is_expanded {
                rgb(gantt.hex)
            } else if done {
                rgba(0xffffff0a)
            } else {
                rgba(0xffffff18)
            })
            .hover(|s| s.bg(rgba(0x1e293be5)).border_color(rgba(0xffffff30)))
            // ── 主条目栏 ─────────────────────────────────────
            .child(
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .px(px(8.0))
                    .py(px(7.0))
                    .gap(px(6.0))
                    .child(div().w(px(3.0)).h(px(20.0)).rounded_full().bg(if done {
                        rgba(0xffffff30)
                    } else {
                        rgb(gantt.hex)
                    }))
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .flex_shrink_0()
                            .rounded_full()
                            .border_2()
                            .cursor_pointer()
                            .id(ElementId::Name(format!("todo-check-{idx}").into()))
                            .border_color(if done {
                                rgb(0x00d992)
                            } else {
                                rgba(0xffffff77)
                            })
                            .bg(if done {
                                rgba(0x00d99230)
                            } else {
                                rgba(0x00000000)
                            })
                            .hover(|s| s.border_color(rgb(0x00d992)))
                            .flex()
                            .justify_center()
                            .items_center()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                on_toggle(this, window, cx, idx);
                            }))
                            .when(done, |d: Stateful<Div>| {
                                d.child(
                                    div()
                                        .text_color(rgb(0x00d992))
                                        .child(Icon::new(IconName::Check).size(px(10.0))),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(1.5))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::NORMAL)
                                    .text_color(if done {
                                        rgba(0x94a3b8aa)
                                    } else {
                                        rgb(0xf8fafc)
                                    })
                                    .when(done, |d: Div| d.line_through())
                                    .child(text),
                            )
                            .when(
                                reminder_text.is_some()
                                    || (item_tag.is_some() && active_tag_id == "all"),
                                |d| {
                                    let mut row = div().flex().items_center().gap(px(4.0));
                                    if active_tag_id == "all" {
                                        if let Some(tag) = &item_tag {
                                            let tag_color =
                                                &GANTT_COLORS[tag.gantt_color % GANTT_COLORS.len()];
                                            row = row.child(
                                                div()
                                                    .px(px(4.0))
                                                    .py(px(0.5))
                                                    .rounded(px(3.0))
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(tag_color.hex))
                                                    .bg(rgba(tag_color.bg_alpha_hex))
                                                    .border_1()
                                                    .border_color(rgba(tag_color.hex | 0x45))
                                                    .child(tag.name.clone()),
                                            );
                                        }
                                    }
                                    if let Some(r_text) = reminder_text {
                                        row = row.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(2.0))
                                                .px(px(4.0))
                                                .py(px(0.5))
                                                .rounded(px(3.0))
                                                .text_xs()
                                                .text_color(rgb(0xfb923c))
                                                .bg(rgba(0xfb923c20))
                                                .child(r_text),
                                        );
                                    }
                                    d.child(row)
                                },
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(if is_expanded {
                                        rgb(gantt.hex)
                                    } else {
                                        rgba(0xffffff60)
                                    })
                                    .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0xffffff)))
                                    .id(ElementId::Name(format!("todo-expand-{idx}").into()))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        on_expand(this, window, cx, idx);
                                    }))
                                    .child(
                                        Icon::new(if is_expanded {
                                            IconName::ChevronUp
                                        } else {
                                            IconName::ChevronDown
                                        })
                                        .size(px(9.0)),
                                    ),
                            )
                            .child(
                                div()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(rgba(0xffffff60))
                                    .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0x00d992)))
                                    .id(ElementId::Name(format!("todo-edit-{idx}").into()))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        on_edit(this, window, cx, idx);
                                    }))
                                    .child(Icon::new(IconName::Redo).size(px(9.0))),
                            )
                            .child(
                                div()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(rgba(0xffffff60))
                                    .hover(|s| s.bg(rgba(0xff4d4d25)).text_color(rgb(0xff6b6b)))
                                    .id(ElementId::Name(format!("todo-del-{idx}").into()))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        on_delete(this, window, cx, idx);
                                    }))
                                    .child(Icon::new(IconName::Delete).size(px(9.0))),
                            ),
                    ),
            )
            // ── 展开面板 ─────────────────────────────────────
            .when(is_expanded, |card| {
                let on_tag = callbacks.on_change_tag.clone();
                let on_rem = callbacks.on_set_reminder.clone();
                let on_col = callbacks.on_set_color.clone();
                card.child(render_item_detail(
                    idx,
                    props.item,
                    &all_tags,
                    move |this, window, cx, idx, tag_id| {
                        on_tag(this, window, cx, idx, tag_id);
                    },
                    move |this, window, cx, idx, rule| {
                        on_rem(this, window, cx, idx, rule);
                    },
                    move |this, window, cx, idx, col| {
                        on_col(this, window, cx, idx, col);
                    },
                    cx,
                ))
            })
            .into_any_element()
    }
}
