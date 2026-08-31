use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};

use crate::model::{TodoTag, GANTT_COLORS};

pub struct ContentPanelProps<'a> {
    pub active_tag_obj: Option<TodoTag>,
    pub pending_count: usize,
    pub can_delete_tag: bool,
    pub new_input: &'a Entity<InputState>,
    pub scroll_handle: &'a ScrollHandle,
    pub show_completed: bool,
    pub pending_elements: Vec<AnyElement>,
    pub completed_elements: Vec<AnyElement>,
}

/// 渲染右侧主体待办毛玻璃面板
pub fn render_content_panel<V: 'static>(
    props: ContentPanelProps,
    on_edit_current_tag: impl Fn(&mut V, &mut Window, &mut Context<V>, TodoTag) + 'static + Clone,
    on_delete_current_tag: impl Fn(&mut V, &mut Window, &mut Context<V>, String) + 'static + Clone,
    on_toggle_completed: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let active_tag_obj = props.active_tag_obj;
    let pending_count = props.pending_count;
    let can_delete_tag = props.can_delete_tag;
    let new_input = props.new_input;
    let scroll_handle = props.scroll_handle;
    let show_completed = props.show_completed;
    let pending_elements = props.pending_elements;
    let completed_elements = props.completed_elements;
    let completed_count = completed_elements.len();

    div()
        .flex()
        .flex_col()
        .flex_1()
        .h_full()
        .bg(rgba(0x0a1220b5))
        .rounded(px(14.0))
        .border_1()
        .border_color(rgba(0xffffff22))
        .p(px(8.0))
        .gap(px(6.0))
        .overflow_hidden()
        // 1. 顶部标题栏（带标签编辑/删除操作区）
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .px(px(4.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(if let Some(t) = &active_tag_obj {
                                    rgb(GANTT_COLORS[t.gantt_color % GANTT_COLORS.len()].hex)
                                } else {
                                    rgb(0x38bdf8)
                                })
                                .child(if let Some(t) = &active_tag_obj {
                                    t.name.clone()
                                } else {
                                    "全部任务".to_string()
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgba(0xffffff50))
                                .child(format!("({} 项)", pending_count)),
                        ),
                )
                // 标题栏右侧操作区：若为特定分类，提供编辑与删除入口
                .child(div().when_some(active_tag_obj.clone(), |d: Div, tag| {
                    let tag_clone_edit = tag.clone();
                    let tag_id_del = tag.id.clone();
                    let on_edit = on_edit_current_tag.clone();
                    let on_delete = on_delete_current_tag.clone();

                    d.flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .w(px(20.0))
                                .h(px(20.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .text_color(rgba(0xffffff60))
                                .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0x38bdf8)))
                                .id("todo-edit-current-tag-btn")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    on_edit(this, window, cx, tag_clone_edit.clone());
                                }))
                                .child(Icon::new(IconName::Redo).size(px(10.0))),
                        )
                        .when(can_delete_tag, |d: Div| {
                            d.child(
                                div()
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(rgba(0xffffff60))
                                    .hover(|s| s.bg(rgba(0xf8717125)).text_color(rgb(0xf87171)))
                                    .id("todo-del-current-tag-btn")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        on_delete(this, window, cx, tag_id_del.clone());
                                    }))
                                    .child(Icon::new(IconName::Delete).size(px(10.0))),
                            )
                        })
                })),
        )
        // 2. 输入框
        .child(
            div()
                .flex()
                .items_center()
                .w_full()
                .px(px(10.0))
                .py(px(8.0))
                .gap(px(6.0))
                .bg(rgba(0x00000040))
                .rounded(px(8.0))
                .border_1()
                .border_color(rgba(0xffffff18))
                .flex_shrink_0()
                .child(
                    div()
                        .w(px(16.0))
                        .h(px(16.0))
                        .flex_shrink_0()
                        .rounded_full()
                        .border_2()
                        .border_color(rgb(0x38bdf8))
                        .flex()
                        .justify_center()
                        .items_center()
                        .text_color(rgb(0x38bdf8))
                        .child(Icon::new(IconName::Plus).size(px(9.0))),
                )
                .child(
                    div()
                        .flex_1()
                        .child(Input::new(new_input).appearance(false).bordered(false)),
                ),
        )
        // 3. 待办条目列表
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .gap(px(5.0))
                .id("todo-list-scroll")
                .track_scroll(scroll_handle)
                .overflow_y_scroll()
                .when(
                    pending_elements.is_empty() && completed_elements.is_empty(),
                    |d| {
                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .py(px(40.0))
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgba(0xffffff35))
                                        .child("暂无待办事项"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgba(0xffffff20))
                                        .child("在上方输入内容即可添加"),
                                ),
                        )
                    },
                )
                .children(pending_elements)
                .when(!completed_elements.is_empty(), |d: Stateful<Div>| {
                    let on_toggle = on_toggle_completed.clone();
                    d.child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .gap(px(4.0))
                            .pt(px(4.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px(px(6.0))
                                    .py(px(3.0))
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgba(0xffffff10)))
                                    .id("todo-toggle-completed")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        on_toggle(this, window, cx);
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(4.0))
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgba(0xffffff77))
                                            .child(
                                                Icon::new(if show_completed {
                                                    IconName::ChevronDown
                                                } else {
                                                    IconName::ChevronRight
                                                })
                                                .size(px(11.0)),
                                            )
                                            .child(format!("已完成 ({})", completed_count)),
                                    ),
                            )
                            .when(show_completed, |d: Div| d.children(completed_elements)),
                    )
                }),
        )
}
