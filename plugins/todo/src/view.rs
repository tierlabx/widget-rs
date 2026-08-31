use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{InputEvent, InputState};

use crate::content_panel::{render_content_panel, ContentPanelProps};
use crate::item_card::{render_todo_item, ItemCardProps};
use crate::model::{TodoData, TodoItem, TodoModel, TodoTag};
use crate::sidebar::render_sidebar;
use crate::tag_modal::{render_tag_modal, TagModalMode, TagModalState};
use crate::timer::{get_now_secs, get_simple_time_str, spawn_todo_timer};

pub struct TodoWidget {
    data: TodoData,
    new_input: Entity<InputState>,
    pending_reset: bool,
    editing_idx: Option<usize>,
    edit_input: Entity<InputState>,
    expanded_idx: Option<usize>,
    show_completed: bool,
    scroll_handle: ScrollHandle,
    tag_modal: Option<TagModalState>,
    _timer: gpui::Task<()>,
}

impl TodoWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = TodoModel::load(cx);
        let new_input = Self::create_new_input(window, cx);
        let edit_input = cx.new(|cx| InputState::new(window, cx).placeholder("编辑待办内容..."));
        let timer = spawn_todo_timer(cx.weak_entity(), cx);

        Self {
            data,
            new_input,
            pending_reset: false,
            editing_idx: None,
            edit_input,
            expanded_idx: None,
            show_completed: false,
            scroll_handle: ScrollHandle::new(),
            tag_modal: None,
            _timer: timer,
        }
    }

    pub fn data(&self) -> &TodoData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut TodoData {
        &mut self.data
    }

    fn create_new_input(window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("输入待办，回车保存..."));
        cx.subscribe(
            &input,
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
        input
    }

    fn open_tag_edit(&mut self, tag: &TodoTag, window: &mut Window, cx: &mut Context<Self>) {
        self.tag_modal = Some(TagModalState::new_edit(tag, window, cx));
        cx.notify();
    }

    fn open_tag_create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tag_modal = Some(TagModalState::new_create(window, cx));
        cx.notify();
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
            self.new_input = Self::create_new_input(window, cx);
        }

        let editing_idx = self.editing_idx;
        let expanded_idx = self.expanded_idx;
        let edit_input = &self.edit_input;
        let new_input = &self.new_input;
        let active_tag_id = self.data.active_tag_id.clone();
        let tags = self.data.tags.clone();
        let active_tag_obj = tags.iter().find(|t| t.id == active_tag_id).cloned();

        let mut pending_elements = Vec::new();
        let mut completed_elements = Vec::new();

        let callbacks = crate::item_card::ItemCardCallbacks {
            on_toggle_done: std::rc::Rc::new(|this: &mut Self, _, cx, idx| {
                if let Some(it) = this.data.items.get_mut(idx) {
                    it.done = !it.done;
                }
                TodoModel::save(&this.data, cx);
                cx.notify();
            }),
            on_toggle_expand: std::rc::Rc::new(|this: &mut Self, _, cx, idx| {
                this.expanded_idx = if this.expanded_idx == Some(idx) {
                    None
                } else {
                    Some(idx)
                };
                cx.notify();
            }),
            on_start_edit: std::rc::Rc::new(|this: &mut Self, window, cx, idx| {
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
                cx.notify();
            }),
            on_confirm_edit: std::rc::Rc::new(|this: &mut Self, _, cx, idx| {
                let text = this.edit_input.read(cx).value().to_string();
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    if let Some(item) = this.data.items.get_mut(idx) {
                        item.text = trimmed;
                    }
                }
                this.editing_idx = None;
                TodoModel::save(&this.data, cx);
                cx.notify();
            }),
            on_delete_item: std::rc::Rc::new(|this: &mut Self, _, cx, idx| {
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
            }),
            on_change_tag: std::rc::Rc::new(|this: &mut Self, _, cx, idx, tag_id| {
                if let Some(it) = this.data.items.get_mut(idx) {
                    it.tag_id = tag_id;
                    TodoModel::save(&this.data, cx);
                    cx.notify();
                }
            }),
            on_set_reminder: std::rc::Rc::new(|this: &mut Self, _, cx, idx, rule| {
                if let Some(it) = this.data.items.get_mut(idx) {
                    it.reminder = rule;
                    TodoModel::save(&this.data, cx);
                    cx.notify();
                }
            }),
            on_set_color: std::rc::Rc::new(|this: &mut Self, _, cx, idx, color_idx| {
                if let Some(it) = this.data.items.get_mut(idx) {
                    it.gantt_color = color_idx;
                    TodoModel::save(&this.data, cx);
                    cx.notify();
                }
            }),
        };

        for (idx, item) in self.data.items.iter().enumerate() {
            if active_tag_id != "all" && item.tag_id != active_tag_id {
                continue;
            }

            let elem = render_todo_item(
                ItemCardProps {
                    idx,
                    item,
                    tags: &tags,
                    active_tag_id: &active_tag_id,
                    is_editing: editing_idx == Some(idx),
                    is_expanded: expanded_idx == Some(idx),
                    edit_input,
                },
                &callbacks,
                cx,
            );

            if item.done {
                completed_elements.push(elem);
            } else {
                pending_elements.push(elem);
            }
        }

        let can_delete_tag = self.data.tags.len() > 1;

        div()
            .relative()
            .flex()
            .flex_row()
            .size_full()
            .gap(px(2.0))
            .overflow_hidden()
            // ── 左侧：吸附 Tab 侧边栏 ──────────────────────────────
            .child(render_sidebar(
                &tags,
                &active_tag_id,
                |this, _, cx, tag_id| {
                    this.data.active_tag_id = tag_id;
                    cx.notify();
                },
                |this, window, cx, tag| {
                    this.open_tag_edit(&tag, window, cx);
                },
                |this, window, cx| {
                    this.open_tag_create(window, cx);
                },
                cx,
            ))
            // ── 右侧：主体内容毛玻璃主面板 ───────────────────────────────
            .child(render_content_panel(
                ContentPanelProps {
                    active_tag_obj,
                    pending_count: pending_elements.len(),
                    can_delete_tag,
                    new_input,
                    scroll_handle: &self.scroll_handle,
                    show_completed: self.show_completed,
                    pending_elements,
                    completed_elements,
                },
                |this, window, cx, tag| {
                    this.open_tag_edit(&tag, window, cx);
                },
                |this, _, cx, tag_id| {
                    this.data.delete_tag_and_migrate(&tag_id);
                    TodoModel::save(&this.data, cx);
                    cx.notify();
                },
                |this, _, cx| {
                    this.show_completed = !this.show_completed;
                    cx.notify();
                },
                cx,
            ))
            // ── 弹窗浮层（标签新建/编辑） ─────────────────────────
            .when_some(self.tag_modal.as_ref(), |d: Div, modal| {
                d.child(render_tag_modal(
                    modal,
                    can_delete_tag,
                    |this, _, cx, color_idx| {
                        if let Some(m) = &mut this.tag_modal {
                            m.selected_color = color_idx;
                            cx.notify();
                        }
                    },
                    |this, _, cx, name, color_idx| {
                        if let Some(m) = &this.tag_modal {
                            match &m.mode {
                                TagModalMode::Create => {
                                    let new_id = this.data.add_tag(name, color_idx);
                                    this.data.active_tag_id = new_id;
                                }
                                TagModalMode::Edit { tag_id } => {
                                    this.data.update_tag(tag_id, name, color_idx);
                                }
                            }
                            TodoModel::save(&this.data, cx);
                        }
                        this.tag_modal = None;
                        cx.notify();
                    },
                    |this, _, cx, tag_id| {
                        this.data.delete_tag_and_migrate(&tag_id);
                        TodoModel::save(&this.data, cx);
                        this.tag_modal = None;
                        cx.notify();
                    },
                    |this, _, cx| {
                        this.tag_modal = None;
                        cx.notify();
                    },
                    cx,
                ))
            })
    }
}
