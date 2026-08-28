use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Icon, IconName};

use crate::model::{TodoItem, TodoModel, GANTT_COLORS};

fn get_simple_time_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let local_secs = (secs + 28800) % 86400;
    let hours = local_secs / 3600;
    let mins = (local_secs % 3600) / 60;
    format!("{:02}:{:02}", hours, mins)
}

#[derive(Clone)]
struct DragTodo(usize);

struct DragTodoView {
    text: String,
}

impl Render for DragTodoView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgba(0x1e2d40cc))
            .border_1()
            .border_color(rgb(0x00d992))
            .rounded(px(6.0))
            .p(px(8.0))
            .text_sm()
            .text_color(rgb(0xf2f2f2))
            .child(self.text.clone())
    }
}

pub struct TodoWidget {
    items: Vec<TodoItem>,
    new_input: Entity<InputState>,
    pending_reset: bool,
    editing_idx: Option<usize>,
    edit_input: Entity<InputState>,
    expanded_idx: Option<usize>,
    show_completed: bool,
    scroll_handle: ScrollHandle,
}

impl TodoWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let saved_items = TodoModel::load(cx);

        let new_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入新待办，回车即保存..."));

        cx.subscribe(
            &new_input,
            |this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    let text = input.read(cx).value().to_string();
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        this.items.push(TodoItem {
                            text: trimmed,
                            done: false,
                            gantt_color: 0,
                            created_at: Some(format!("今日 {}", get_simple_time_str())),
                        });
                        TodoModel::save(&this.items, cx);
                        this.scroll_handle.scroll_to_bottom();
                    }
                    this.pending_reset = true;
                    cx.notify();
                }
            },
        )
        .detach();

        let edit_input = cx.new(|cx| InputState::new(window, cx).placeholder("编辑待办内容..."));

        Self {
            items: saved_items,
            new_input,
            pending_reset: false,
            editing_idx: None,
            edit_input,
            expanded_idx: None,
            show_completed: false,
            scroll_handle: ScrollHandle::new(),
        }
    }
}

impl widget_core::WidgetContent for TodoWidget {
    fn plugin_id(&self) -> &'static str {
        "todo_widget"
    }

    fn drag_label(&self) -> &'static str {
        "拖拽移动待办"
    }
}

impl Render for TodoWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui_component::Theme::global_mut(cx).colors.foreground = gpui::hsla(0.0, 0.0, 0.98, 1.0);
        gpui_component::Theme::global_mut(cx)
            .colors
            .muted_foreground = gpui::hsla(0.0, 0.0, 0.65, 1.0);

        if self.pending_reset {
            self.pending_reset = false;
            let new_entity =
                cx.new(|cx| InputState::new(window, cx).placeholder("输入新待办，回车即保存..."));
            cx.subscribe(
                &new_entity,
                |this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                    if let InputEvent::PressEnter { .. } = event {
                        let text = input.read(cx).value().to_string();
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            this.items.push(TodoItem {
                                text: trimmed,
                                done: false,
                                gantt_color: 0,
                                created_at: Some(format!("今日 {}", get_simple_time_str())),
                            });
                            TodoModel::save(&this.items, cx);
                            this.scroll_handle.scroll_to_bottom();
                        }
                        this.pending_reset = true;
                        cx.notify();
                    }
                },
            )
            .detach();
            self.new_input = new_entity;
        }

        let editing_idx = self.editing_idx;
        let expanded_idx = self.expanded_idx;
        let edit_input = &self.edit_input;
        let new_input = &self.new_input;

        let mut pending_elements = Vec::new();
        let mut completed_elements = Vec::new();

        for (idx, item) in self.items.iter().enumerate() {
            let done = item.done;
            let text = item.text.clone();
            let is_editing = editing_idx == Some(idx);
            let is_expanded = expanded_idx == Some(idx);
            let color_idx = item.gantt_color % GANTT_COLORS.len();
            let gantt = &GANTT_COLORS[color_idx];
            let created_time = item
                .created_at
                .clone()
                .unwrap_or_else(|| "今天".to_string());

            let toggle_handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
                if let Some(it) = this.items.get_mut(idx) {
                    it.done = !it.done;
                }
                TodoModel::save(&this.items, cx);
                cx.notify();
            });

            let delete_handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
                if this.editing_idx == Some(idx) {
                    this.editing_idx = None;
                }
                if this.expanded_idx == Some(idx) {
                    this.expanded_idx = None;
                }
                if idx < this.items.len() {
                    this.items.remove(idx);
                }
                TodoModel::save(&this.items, cx);
                cx.notify();
            });

            let edit_handler = cx.listener(move |this, _: &ClickEvent, window, cx| {
                if this.editing_idx == Some(idx) {
                    let text = this.edit_input.read(cx).value().to_string();
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        if let Some(item) = this.items.get_mut(idx) {
                            item.text = trimmed;
                        }
                    }
                    this.editing_idx = None;
                    TodoModel::save(&this.items, cx);
                } else {
                    let current = this.items[idx].text.clone();
                    let new_edit = cx.new(|cx| {
                        InputState::new(window, cx)
                            .default_value(current)
                            .placeholder("编辑待办内容，Enter 确认...")
                    });
                    cx.subscribe(
                        &new_edit,
                        |this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                            if let InputEvent::PressEnter { .. } = event {
                                let text = input.read(cx).value().to_string();
                                let trimmed = text.trim().to_string();
                                if let Some(idx) = this.editing_idx {
                                    if !trimmed.is_empty() {
                                        if let Some(item) = this.items.get_mut(idx) {
                                            item.text = trimmed;
                                        }
                                    }
                                }
                                this.editing_idx = None;
                                TodoModel::save(&this.items, cx);
                                cx.notify();
                            }
                        },
                    )
                    .detach();
                    this.edit_input = new_edit;
                    this.editing_idx = Some(idx);
                }
                cx.notify();
            });

            let item_element = if is_editing {
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .px(px(10.0))
                    .py(px(8.0))
                    .gap(px(8.0))
                    .bg(rgba(0x0f172af0))
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(rgb(0x38bdf8))
                    .child(
                        div()
                            .flex_1()
                            .child(Input::new(edit_input).appearance(false).bordered(false)),
                    )
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .bg(rgba(0x00d99220))
                            .text_color(rgb(0x00d992))
                            .hover(|s| s.bg(rgba(0x00d99245)))
                            .id(ElementId::Name(format!("todo-confirm-{idx}").into()))
                            .on_click(edit_handler)
                            .child(Icon::new(IconName::Check).size(px(12.0))),
                    )
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .bg(rgba(0xff4d4d20))
                            .text_color(rgb(0xff6b6b))
                            .hover(|s| s.bg(rgba(0xff4d4d40)))
                            .id(ElementId::Name(format!("todo-del-{idx}").into()))
                            .on_click(delete_handler)
                            .child(Icon::new(IconName::Delete).size(px(12.0))),
                    )
                    .into_any_element()
            } else {
                let drag_text = text.clone();
                let item_text_display = text.clone();

                div()
                    .id(ElementId::Name(format!("todo-item-{idx}").into()))
                    .on_drag(DragTodo(idx), move |_, _, _, cx| {
                        cx.new(|_| DragTodoView {
                            text: drag_text.clone(),
                        })
                    })
                    .on_drop(cx.listener(move |this, drag: &DragTodo, _, cx| {
                        let from = drag.0;
                        let to = idx;
                        if from != to && from < this.items.len() && to < this.items.len() {
                            let item = this.items.remove(from);
                            let adjusted_to = if from < to { to - 1 } else { to };
                            this.items.insert(adjusted_to, item);
                            TodoModel::save(&this.items, cx);
                            cx.notify();
                        }
                    }))
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
                            .py(px(8.0))
                            .gap(px(8.0))
                            // 1. 甘特色系竖条（Gantt Bar）
                            .child(div().w(px(3.5)).h(px(20.0)).rounded_full().bg(if done {
                                rgba(0xffffff30)
                            } else {
                                rgb(gantt.hex)
                            }))
                            // 2. 勾选圈
                            .child(
                                div()
                                    .w(px(18.0))
                                    .h(px(18.0))
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
                                    .on_click(toggle_handler)
                                    .when(done, |d: Stateful<Div>| {
                                        d.child(
                                            div()
                                                .text_color(rgb(0x00d992))
                                                .child(Icon::new(IconName::Check).size(px(11.0))),
                                        )
                                    }),
                            )
                            // 3. 待办文本
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .font_weight(FontWeight::NORMAL)
                                    .text_color(if done {
                                        rgba(0x94a3b8aa)
                                    } else {
                                        rgb(0xf8fafc)
                                    })
                                    .when(done, |d: Div| d.line_through())
                                    .child(item_text_display),
                            )
                            // 4. 右侧操作区（展开详情、编辑、删除）
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(2.0))
                                    // 展开/折叠更多详情按钮
                                    .child(
                                        div()
                                            .w(px(20.0))
                                            .h(px(20.0))
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
                                            .hover(|s| {
                                                s.bg(rgba(0xffffff15)).text_color(rgb(0xffffff))
                                            })
                                            .id(ElementId::Name(
                                                format!("todo-expand-{idx}").into(),
                                            ))
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if this.expanded_idx == Some(idx) {
                                                    this.expanded_idx = None;
                                                } else {
                                                    this.expanded_idx = Some(idx);
                                                }
                                                cx.notify();
                                            }))
                                            .child(
                                                Icon::new(if is_expanded {
                                                    IconName::ChevronUp
                                                } else {
                                                    IconName::ChevronDown
                                                })
                                                .size(px(10.0)),
                                            ),
                                    )
                                    // 编辑
                                    .child(
                                        div()
                                            .w(px(20.0))
                                            .h(px(20.0))
                                            .flex()
                                            .justify_center()
                                            .items_center()
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .text_color(rgba(0xffffff60))
                                            .hover(|s| {
                                                s.bg(rgba(0xffffff15)).text_color(rgb(0x00d992))
                                            })
                                            .id(ElementId::Name(format!("todo-edit-{idx}").into()))
                                            .on_click(edit_handler)
                                            .child(Icon::new(IconName::Redo).size(px(10.0))),
                                    )
                                    // 删除
                                    .child(
                                        div()
                                            .w(px(20.0))
                                            .h(px(20.0))
                                            .flex()
                                            .justify_center()
                                            .items_center()
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .text_color(rgba(0xffffff60))
                                            .hover(|s| {
                                                s.bg(rgba(0xff4d4d25)).text_color(rgb(0xff6b6b))
                                            })
                                            .id(ElementId::Name(format!("todo-del-{idx}").into()))
                                            .on_click(delete_handler)
                                            .child(Icon::new(IconName::Delete).size(px(10.0))),
                                    ),
                            ),
                    )
                    // ── 鼠标移入/点击展开的“更多内容”卡片 ───────────────
                    .when(is_expanded, |card| {
                        card.child(
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .px(px(10.0))
                                .py(px(6.0))
                                .gap(px(6.0))
                                .bg(rgba(0x00000030))
                                .border_t_1()
                                .border_color(rgba(0xffffff10))
                                // 时间与分类标签
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .w_full()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgba(0xffffff50))
                                                .child(format!("⏱ {}", created_time)),
                                        )
                                        .child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(1.5))
                                                .rounded_full()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(gantt.hex))
                                                .bg(rgba(gantt.bg_alpha_hex))
                                                .child(gantt.name),
                                        ),
                                )
                                // 甘特色系快速切换栏
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgba(0xffffff60))
                                                .child("甘特色:"),
                                        )
                                        .children(GANTT_COLORS.iter().enumerate().map(
                                            |(g_idx, g_color)| {
                                                let is_curr = g_idx == color_idx;
                                                div()
                                                    .w(px(14.0))
                                                    .h(px(14.0))
                                                    .rounded_full()
                                                    .cursor_pointer()
                                                    .bg(rgb(g_color.hex))
                                                    .border_2()
                                                    .border_color(if is_curr {
                                                        rgb(0xffffff)
                                                    } else {
                                                        rgba(0x00000000)
                                                    })
                                                    .hover(|s| s.border_color(rgb(0xffffff)))
                                                    .id(ElementId::Name(
                                                        format!("todo-gantt-{idx}-{g_idx}").into(),
                                                    ))
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        if let Some(it) = this.items.get_mut(idx) {
                                                            it.gantt_color = g_idx;
                                                            TodoModel::save(&this.items, cx);
                                                            cx.notify();
                                                        }
                                                    }))
                                            },
                                        )),
                                ),
                        )
                    })
                    .into_any_element()
            };

            if done {
                completed_elements.push(item_element);
            } else {
                pending_elements.push(item_element);
            }
        }

        // ── 整体布局（高质感深海蓝黑半透明毛玻璃底板）──────────────────
        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .bg(rgba(0x0a1220b5))
            .rounded(px(14.0))
            .border_1()
            .border_color(rgba(0xffffff22))
            .p(px(8.0))
            .gap(px(6.0))
            .overflow_hidden()
            // 1. 顶部常驻新增输入栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .px(px(12.0))
                    .py(px(9.0))
                    .gap(px(8.0))
                    .bg(rgba(0x00000040))
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(rgba(0xffffff18))
                    .flex_shrink_0()
                    .child(
                        div()
                            .w(px(18.0))
                            .h(px(18.0))
                            .flex_shrink_0()
                            .rounded_full()
                            .border_2()
                            .border_color(rgb(0x38bdf8))
                            .flex()
                            .justify_center()
                            .items_center()
                            .text_color(rgb(0x38bdf8))
                            .child(Icon::new(IconName::Plus).size(px(10.0))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(Input::new(new_input).appearance(false).bordered(false)),
                    ),
            )
            // 2. 待办条目列表
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .w_full()
                    .gap(px(5.0))
                    .id("todo-list-scroll")
                    .track_scroll(&self.scroll_handle)
                    .overflow_y_scroll()
                    // 未完成列表
                    .children(pending_elements)
                    // 已完成折叠区
                    .when(!completed_elements.is_empty(), |d: Stateful<Div>| {
                        let completed_count = completed_elements.len();
                        let show = self.show_completed;

                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .gap(px(4.0))
                                .pt(px(6.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px(px(6.0))
                                        .py(px(4.0))
                                        .rounded(px(6.0))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgba(0xffffff10)))
                                        .id("todo-toggle-completed")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.show_completed = !this.show_completed;
                                            cx.notify();
                                        }))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgba(0xffffff77))
                                                .child(
                                                    Icon::new(if show {
                                                        IconName::ChevronDown
                                                    } else {
                                                        IconName::ChevronRight
                                                    })
                                                    .size(px(12.0)),
                                                )
                                                .child(format!("已完成 ({})", completed_count)),
                                        ),
                                )
                                .when(show, |d: Div| d.children(completed_elements)),
                        )
                    }),
            )
    }
}
