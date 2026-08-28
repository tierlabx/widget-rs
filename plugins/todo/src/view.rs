use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Icon, IconName};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::model::{ReminderRule, TodoData, TodoItem, TodoModel, TodoTag, GANTT_COLORS};

fn get_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn get_simple_time_str() -> String {
    let secs = get_now_secs();
    let local_secs = (secs + 28800) % 86400; // UTC+8
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
    data: TodoData,
    new_input: Entity<InputState>,
    pending_reset: bool,
    editing_idx: Option<usize>,
    edit_input: Entity<InputState>,
    expanded_idx: Option<usize>,
    show_completed: bool,
    scroll_handle: ScrollHandle,
    _timer: gpui::Task<()>,
}

impl TodoWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = TodoModel::load(cx);

        let new_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入待办，回车保存..."));

        cx.subscribe(
            &new_input,
            |this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    let text = input.read(cx).value().to_string();
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        let active_tag = if this.data.active_tag_id == "all" {
                            this.data
                                .tags
                                .first()
                                .map(|t| t.id.clone())
                                .unwrap_or_else(|| "work".to_string())
                        } else {
                            this.data.active_tag_id.clone()
                        };

                        this.data.items.push(TodoItem {
                            id: format!("todo-{}", get_now_secs()),
                            text: trimmed,
                            done: false,
                            tag_id: active_tag,
                            gantt_color: 0,
                            reminder: None,
                            last_reminded_at: None,
                            created_at: Some(format!("今日 {}", get_simple_time_str())),
                        });
                        TodoModel::save(&this.data, cx);
                        this.scroll_handle.scroll_to_bottom();
                    }
                    this.pending_reset = true;
                    cx.notify();
                }
            },
        )
        .detach();

        let edit_input = cx.new(|cx| InputState::new(window, cx).placeholder("编辑待办内容..."));

        let this_weak = cx.weak_entity();
        let app_cx: &mut App = cx;
        let _timer = app_cx.spawn(async move |async_cx| loop {
            async_cx
                .background_executor()
                .timer(Duration::from_secs(2))
                .await;

            let res = async_cx.update(|cx| {
                let _ = this_weak.update(cx, |this, cx| {
                    let now = get_now_secs();
                    let local_secs = (now + 28800) % 86400;
                    let curr_minute_of_day = (local_secs / 60) as u32;

                    let mut needs_save = false;
                    for item in &mut this.data.items {
                        if item.done {
                            continue;
                        }
                        if let Some(rule) = &item.reminder {
                            let should_remind = match rule {
                                ReminderRule::Once { target_time_secs } => {
                                    now >= *target_time_secs && item.last_reminded_at.is_none()
                                }
                                ReminderRule::Daily { minute_of_day } => {
                                    curr_minute_of_day == *minute_of_day
                                        && item
                                            .last_reminded_at
                                            .map(|t| now.saturating_sub(t) > 60)
                                            .unwrap_or(true)
                                }
                                ReminderRule::Weekly { minute_of_day, .. } => {
                                    curr_minute_of_day == *minute_of_day
                                        && item
                                            .last_reminded_at
                                            .map(|t| now.saturating_sub(t) > 60)
                                            .unwrap_or(true)
                                }
                                ReminderRule::Monthly { minute_of_day, .. } => {
                                    curr_minute_of_day == *minute_of_day
                                        && item
                                            .last_reminded_at
                                            .map(|t| now.saturating_sub(t) > 60)
                                            .unwrap_or(true)
                                }
                                ReminderRule::Interval { interval_mins } => item
                                    .last_reminded_at
                                    .map(|t| now.saturating_sub(t) >= (*interval_mins as u64 * 60))
                                    .unwrap_or(true),
                            };

                            if should_remind {
                                item.last_reminded_at = Some(now);
                                needs_save = true;
                            }
                        }
                    }
                    if needs_save {
                        TodoModel::save(&this.data, cx);
                        cx.notify();
                    }
                });
            });

            if res.is_err() {
                break;
            }
        });

        Self {
            data,
            new_input,
            pending_reset: false,
            editing_idx: None,
            edit_input,
            expanded_idx: None,
            show_completed: false,
            scroll_handle: ScrollHandle::new(),
            _timer,
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
                cx.new(|cx| InputState::new(window, cx).placeholder("输入待办，回车保存..."));
            cx.subscribe(
                &new_entity,
                |this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                    if let InputEvent::PressEnter { .. } = event {
                        let text = input.read(cx).value().to_string();
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            let active_tag = if this.data.active_tag_id == "all" {
                                this.data
                                    .tags
                                    .first()
                                    .map(|t| t.id.clone())
                                    .unwrap_or_else(|| "work".to_string())
                            } else {
                                this.data.active_tag_id.clone()
                            };

                            this.data.items.push(TodoItem {
                                id: format!("todo-{}", get_now_secs()),
                                text: trimmed,
                                done: false,
                                tag_id: active_tag,
                                gantt_color: 0,
                                reminder: None,
                                last_reminded_at: None,
                                created_at: Some(format!("今日 {}", get_simple_time_str())),
                            });
                            TodoModel::save(&this.data, cx);
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
        let active_tag_id = self.data.active_tag_id.clone();

        let mut pending_elements = Vec::new();
        let mut completed_elements = Vec::new();

        let tags = self.data.tags.clone();
        let active_tag_obj = tags.iter().find(|t| t.id == active_tag_id).cloned();

        for (idx, item) in self.data.items.iter().enumerate() {
            if active_tag_id != "all" && item.tag_id != active_tag_id {
                continue;
            }

            let done = item.done;
            let text = item.text.clone();
            let is_editing = editing_idx == Some(idx);
            let is_expanded = expanded_idx == Some(idx);
            let color_idx = item.gantt_color % GANTT_COLORS.len();
            let gantt = &GANTT_COLORS[color_idx];
            let item_tag = tags.iter().find(|t| t.id == item.tag_id).cloned();
            let reminder_text = item.reminder.as_ref().map(|r| r.display_text());
            let created_time = item
                .created_at
                .clone()
                .unwrap_or_else(|| "今天".to_string());

            let toggle_handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
                if let Some(it) = this.data.items.get_mut(idx) {
                    it.done = !it.done;
                }
                TodoModel::save(&this.data, cx);
                cx.notify();
            });

            let delete_handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
                if this.editing_idx == Some(idx) {
                    this.editing_idx = None;
                }
                if this.expanded_idx == Some(idx) {
                    this.expanded_idx = None;
                }
                if idx < this.data.items.len() {
                    this.data.items.remove(idx);
                }
                TodoModel::save(&this.data, cx);
                cx.notify();
            });

            let edit_handler = cx.listener(move |this, _: &ClickEvent, window, cx| {
                if this.editing_idx == Some(idx) {
                    let text = this.edit_input.read(cx).value().to_string();
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        if let Some(item) = this.data.items.get_mut(idx) {
                            item.text = trimmed;
                        }
                    }
                    this.editing_idx = None;
                    TodoModel::save(&this.data, cx);
                } else {
                    let current = this.data.items[idx].text.clone();
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
                                        if let Some(item) = this.data.items.get_mut(idx) {
                                            item.text = trimmed;
                                        }
                                    }
                                }
                                this.editing_idx = None;
                                TodoModel::save(&this.data, cx);
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
                    .px(px(8.0))
                    .py(px(6.0))
                    .gap(px(6.0))
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
                            .on_click(edit_handler)
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
                            .on_click(delete_handler)
                            .child(Icon::new(IconName::Delete).size(px(11.0))),
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
                        if from != to && from < this.data.items.len() && to < this.data.items.len()
                        {
                            let item = this.data.items.remove(from);
                            let adjusted_to = if from < to { to - 1 } else { to };
                            this.data.items.insert(adjusted_to, item);
                            TodoModel::save(&this.data, cx);
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
                            .py(px(7.0))
                            .gap(px(6.0))
                            // 1. 甘特色系竖条
                            .child(div().w(px(3.0)).h(px(20.0)).rounded_full().bg(if done {
                                rgba(0xffffff30)
                            } else {
                                rgb(gantt.hex)
                            }))
                            // 2. 勾选圈
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
                                    .on_click(toggle_handler)
                                    .when(done, |d: Stateful<Div>| {
                                        d.child(
                                            div()
                                                .text_color(rgb(0x00d992))
                                                .child(Icon::new(IconName::Check).size(px(10.0))),
                                        )
                                    }),
                            )
                            // 3. 待办文本与提醒徽章
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
                                            .child(item_text_display),
                                    )
                                    .when(
                                        reminder_text.is_some()
                                            || (item_tag.is_some() && active_tag_id == "all"),
                                        |d| {
                                            let mut row = div().flex().items_center().gap(px(4.0));
                                            if active_tag_id == "all" {
                                                if let Some(tag) = &item_tag {
                                                    let tag_color = &GANTT_COLORS
                                                        [tag.gantt_color % GANTT_COLORS.len()];
                                                    row = row.child(
                                                        div()
                                                            .px(px(4.0))
                                                            .py(px(0.5))
                                                            .rounded(px(3.0))
                                                            .text_xs()
                                                            .text_color(rgb(tag_color.hex))
                                                            .bg(rgba(tag_color.bg_alpha_hex))
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
                                                        .child(format!("⏰ {}", r_text)),
                                                );
                                            }
                                            d.child(row)
                                        },
                                    ),
                            )
                            // 4. 右侧操作区
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
                                            .hover(|s| {
                                                s.bg(rgba(0xffffff15)).text_color(rgb(0x00d992))
                                            })
                                            .id(ElementId::Name(format!("todo-edit-{idx}").into()))
                                            .on_click(edit_handler)
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
                                            .hover(|s| {
                                                s.bg(rgba(0xff4d4d25)).text_color(rgb(0xff6b6b))
                                            })
                                            .id(ElementId::Name(format!("todo-del-{idx}").into()))
                                            .on_click(delete_handler)
                                            .child(Icon::new(IconName::Delete).size(px(9.0))),
                                    ),
                            ),
                    )
                    // ── 展开面板 ─────────────────────────────────────
                    .when(is_expanded, |card| {
                        let all_tags = tags.clone();
                        card.child(
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .px(px(8.0))
                                .py(px(6.0))
                                .gap(px(6.0))
                                .bg(rgba(0x00000035))
                                .border_t_1()
                                .border_color(rgba(0xffffff10))
                                // 1. 分类标签选择
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(4.0))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgba(0xffffff60))
                                                .child("分类:"),
                                        )
                                        .children(all_tags.iter().map(|tag| {
                                            let is_curr = item.tag_id == tag.id;
                                            let tag_id_clone = tag.id.clone();
                                            let tag_color =
                                                &GANTT_COLORS[tag.gantt_color % GANTT_COLORS.len()];
                                            div()
                                                .px(px(5.0))
                                                .py(px(1.0))
                                                .rounded(px(3.0))
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(if is_curr {
                                                    rgb(0xffffff)
                                                } else {
                                                    rgb(tag_color.hex)
                                                })
                                                .bg(if is_curr {
                                                    rgb(tag_color.hex)
                                                } else {
                                                    rgba(tag_color.bg_alpha_hex)
                                                })
                                                .hover(|s| s.opacity(0.8))
                                                .id(ElementId::Name(
                                                    format!("todo-tag-{idx}-{}", tag.id).into(),
                                                ))
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    if let Some(it) = this.data.items.get_mut(idx) {
                                                        it.tag_id = tag_id_clone.clone();
                                                        TodoModel::save(&this.data, cx);
                                                        cx.notify();
                                                    }
                                                }))
                                                .child(tag.name.clone())
                                        })),
                                )
                                // 2. 提醒设置
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(3.0))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgba(0xffffff60))
                                                .child("提醒设置:"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_wrap()
                                                .gap(px(3.0))
                                                .child(
                                                    div()
                                                        .px(px(5.0))
                                                        .py(px(1.5))
                                                        .rounded(px(3.0))
                                                        .cursor_pointer()
                                                        .text_xs()
                                                        .text_color(if item.reminder.is_none() {
                                                            rgb(0xffffff)
                                                        } else {
                                                            rgba(0xffffff60)
                                                        })
                                                        .bg(if item.reminder.is_none() {
                                                            rgba(0x38bdf840)
                                                        } else {
                                                            rgba(0xffffff10)
                                                        })
                                                        .id(ElementId::Name(
                                                            format!("todo-rem-none-{idx}").into(),
                                                        ))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                if let Some(it) =
                                                                    this.data.items.get_mut(idx)
                                                                {
                                                                    it.reminder = None;
                                                                    TodoModel::save(&this.data, cx);
                                                                    cx.notify();
                                                                }
                                                            },
                                                        ))
                                                        .child("无"),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(5.0))
                                                        .py(px(1.5))
                                                        .rounded(px(3.0))
                                                        .cursor_pointer()
                                                        .text_xs()
                                                        .text_color(rgb(0x38bdf8))
                                                        .bg(rgba(0x38bdf818))
                                                        .hover(|s| s.bg(rgba(0x38bdf835)))
                                                        .id(ElementId::Name(
                                                            format!("todo-rem-30m-{idx}").into(),
                                                        ))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                if let Some(it) =
                                                                    this.data.items.get_mut(idx)
                                                                {
                                                                    it.reminder =
                                                                        Some(ReminderRule::Once {
                                                                            target_time_secs:
                                                                                get_now_secs()
                                                                                    + 30 * 60,
                                                                        });
                                                                    TodoModel::save(&this.data, cx);
                                                                    cx.notify();
                                                                }
                                                            },
                                                        ))
                                                        .child("30分钟后"),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(5.0))
                                                        .py(px(1.5))
                                                        .rounded(px(3.0))
                                                        .cursor_pointer()
                                                        .text_xs()
                                                        .text_color(rgb(0x34d399))
                                                        .bg(rgba(0x34d39918))
                                                        .hover(|s| s.bg(rgba(0x34d39935)))
                                                        .id(ElementId::Name(
                                                            format!("todo-rem-daily-{idx}").into(),
                                                        ))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                if let Some(it) =
                                                                    this.data.items.get_mut(idx)
                                                                {
                                                                    it.reminder =
                                                                        Some(ReminderRule::Daily {
                                                                            minute_of_day: 18 * 60,
                                                                        });
                                                                    TodoModel::save(&this.data, cx);
                                                                    cx.notify();
                                                                }
                                                            },
                                                        ))
                                                        .child("每天 18:00"),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(5.0))
                                                        .py(px(1.5))
                                                        .rounded(px(3.0))
                                                        .cursor_pointer()
                                                        .text_xs()
                                                        .text_color(rgb(0xa78bfa))
                                                        .bg(rgba(0xa78bfa18))
                                                        .hover(|s| s.bg(rgba(0xa78bfa35)))
                                                        .id(ElementId::Name(
                                                            format!("todo-rem-weekly-{idx}").into(),
                                                        ))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                if let Some(it) =
                                                                    this.data.items.get_mut(idx)
                                                                {
                                                                    it.reminder = Some(
                                                                        ReminderRule::Weekly {
                                                                            weekday: 5,
                                                                            minute_of_day: 17 * 60,
                                                                        },
                                                                    );
                                                                    TodoModel::save(&this.data, cx);
                                                                    cx.notify();
                                                                }
                                                            },
                                                        ))
                                                        .child("周五 17:00"),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(5.0))
                                                        .py(px(1.5))
                                                        .rounded(px(3.0))
                                                        .cursor_pointer()
                                                        .text_xs()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(0xfb923c))
                                                        .bg(rgba(0xfb923c20))
                                                        .hover(|s| s.bg(rgba(0xfb923c40)))
                                                        .id(ElementId::Name(
                                                            format!("todo-rem-loop30-{idx}").into(),
                                                        ))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                if let Some(it) =
                                                                    this.data.items.get_mut(idx)
                                                                {
                                                                    it.reminder = Some(
                                                                        ReminderRule::Interval {
                                                                            interval_mins: 30,
                                                                        },
                                                                    );
                                                                    TodoModel::save(&this.data, cx);
                                                                    cx.notify();
                                                                }
                                                            },
                                                        ))
                                                        .child("⚡ 每30分催办"),
                                                ),
                                        ),
                                )
                                // 3. 甘特色系
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .w_full()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(4.0))
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
                                                            .w(px(12.0))
                                                            .h(px(12.0))
                                                            .rounded_full()
                                                            .cursor_pointer()
                                                            .bg(rgb(g_color.hex))
                                                            .border_2()
                                                            .border_color(if is_curr {
                                                                rgb(0xffffff)
                                                            } else {
                                                                rgba(0x00000000)
                                                            })
                                                            .hover(|s| {
                                                                s.border_color(rgb(0xffffff))
                                                            })
                                                            .id(ElementId::Name(
                                                                format!("todo-gantt-{idx}-{g_idx}")
                                                                    .into(),
                                                            ))
                                                            .on_click(cx.listener(
                                                                move |this, _, _, cx| {
                                                                    if let Some(it) =
                                                                        this.data.items.get_mut(idx)
                                                                    {
                                                                        it.gantt_color = g_idx;
                                                                        TodoModel::save(
                                                                            &this.data, cx,
                                                                        );
                                                                        cx.notify();
                                                                    }
                                                                },
                                                            ))
                                                    },
                                                )),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgba(0xffffff40))
                                                .child(created_time),
                                        ),
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

        // ══════════════════════════════════════════════════════════════════════
        // 整体双栏布局：左侧敬业签吸附 Tab 侧栏 + 右侧毛玻璃主面板
        // ══════════════════════════════════════════════════════════════════════
        div()
            .flex()
            .flex_row()
            .size_full()
            .gap(px(2.0))
            .overflow_hidden()
            // ── 左侧：纵向吸附 Tab 侧边栏 ──────────────────────────────
            .child(
                div()
                    .w(px(46.0))
                    .flex()
                    .flex_col()
                    .gap(px(3.0))
                    .pt(px(16.0))
                    .pb(px(8.0))
                    .overflow_hidden()
                    // 1. "全部" 标签
                    .child({
                        let is_active = active_tag_id == "all";
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
                                rgb(0xffffff)
                            } else {
                                rgba(0xffffff80)
                            })
                            .bg(if is_active {
                                rgb(0x38bdf8)
                            } else {
                                rgba(0x0f172a65)
                            })
                            .border_1()
                            .border_color(if is_active {
                                rgb(0x7dd3fc)
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
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.data.active_tag_id = "all".to_string();
                                cx.notify();
                            }))
                            .child("全部")
                    })
                    // 2. 自定义分类标签列表
                    .children(tags.iter().map(|tag| {
                        let is_active = active_tag_id == tag.id;
                        let tag_id_clone = tag.id.clone();
                        let tag_color = &GANTT_COLORS[tag.gantt_color % GANTT_COLORS.len()];

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
                                rgb(0xffffff)
                            } else {
                                rgba(0xffffff85)
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
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.data.active_tag_id = tag_id_clone.clone();
                                cx.notify();
                            }))
                            // 左侧微型甘特色点
                            .child(
                                div()
                                    .absolute()
                                    .left(px(3.0))
                                    .w(px(4.0))
                                    .h(px(4.0))
                                    .rounded_full()
                                    .bg(if is_active {
                                        rgb(0xffffff)
                                    } else {
                                        rgb(tag_color.hex)
                                    }),
                            )
                            .child(tag.name.clone())
                    }))
                    // 3. 底部 "+" 快速添加标签按钮
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
                            .on_click(cx.listener(|this, _, _, cx| {
                                let new_id = format!("tag-{}", get_now_secs());
                                let tag_idx = this.data.tags.len();
                                this.data.tags.push(TodoTag {
                                    id: new_id.clone(),
                                    name: format!("分类{}", tag_idx + 1),
                                    gantt_color: tag_idx % GANTT_COLORS.len(),
                                });
                                this.data.active_tag_id = new_id;
                                TodoModel::save(&this.data, cx);
                                cx.notify();
                            }))
                            .child(Icon::new(IconName::Plus).size(px(12.0))),
                    ),
            )
            // ── 右侧：主体内容毛玻璃主面板 ───────────────────────────────
            .child(
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
                    // 1. 面板顶部标题栏（当前分类徽标）
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
                                                rgb(GANTT_COLORS
                                                    [t.gantt_color % GANTT_COLORS.len()]
                                                .hex)
                                            } else {
                                                rgb(0x38bdf8)
                                            })
                                            .child(if let Some(t) = &active_tag_obj {
                                                format!("🏷️ {}", t.name)
                                            } else {
                                                "🏷️ 全部任务".to_string()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgba(0xffffff50))
                                            .child(format!("({} 项)", pending_elements.len())),
                                    ),
                            ),
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
                            .track_scroll(&self.scroll_handle)
                            .overflow_y_scroll()
                            .children(pending_elements)
                            .when(!completed_elements.is_empty(), |d: Stateful<Div>| {
                                let completed_count = completed_elements.len();
                                let show = self.show_completed;

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
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.show_completed = !this.show_completed;
                                                    cx.notify();
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
                                                            Icon::new(if show {
                                                                IconName::ChevronDown
                                                            } else {
                                                                IconName::ChevronRight
                                                            })
                                                            .size(px(11.0)),
                                                        )
                                                        .child(format!(
                                                            "已完成 ({})",
                                                            completed_count
                                                        )),
                                                ),
                                        )
                                        .when(show, |d: Div| d.children(completed_elements)),
                                )
                            }),
                    ),
            )
    }
}
