use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{InputEvent, InputState};

use crate::model::{TodoData, TodoItem, TodoModel, TodoTag};
use crate::timer::{get_now_millis, get_simple_time_str, spawn_todo_timer};
use crate::ui::content_panel::{render_content_panel, ContentPanelProps};
use crate::ui::item_card::{render_todo_item, ItemCardProps};
use crate::ui::sidebar::{render_sidebar, SidebarProps};
use crate::ui::tag_modal::{render_tag_modal, TagModalMode, TagModalProps, TagModalState};

pub struct TodoWidget {
    data: TodoData,
    new_input: Entity<InputState>,
    _new_input_sub: gpui::Subscription,
    editing_idx: Option<usize>,
    edit_input: Entity<InputState>,
    _edit_input_sub: gpui::Subscription,
    tag_name_input: Entity<InputState>,
    expanded_idx: Option<usize>,
    show_completed: bool,
    scroll_handle: ScrollHandle,
    sidebar_scroll_handle: ScrollHandle,
    tag_modal: Option<TagModalState>,
    _timer: gpui::Task<()>,
}

impl TodoWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = TodoModel::load(cx);
        let new_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入待办，回车保存..."));
        let new_input_sub = cx.subscribe_in(
            &new_input,
            window,
            |this: &mut Self,
             input: &Entity<InputState>,
             event: &InputEvent,
             window: &mut Window,
             cx: &mut Context<Self>| {
                if let InputEvent::PressEnter { shift: false, .. } = event {
                    let text = input.read(cx).value().to_string();
                    let trimmed = text.trim().to_string();
                    // 立即清空输入框内容，防止重复事件或重入导致二次添加
                    input.update(cx, |state, cx| {
                        state.set_value("", window, cx);
                    });
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
                            id: format!("todo-{}", get_now_millis()),
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
                        cx.notify();
                    }
                }
            },
        );
        let edit_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("编辑待办内容，Enter 确认..."));
        let edit_input_sub = cx.subscribe_in(
            &edit_input,
            window,
            |this: &mut Self,
             _input: &Entity<InputState>,
             event: &InputEvent,
             window: &mut Window,
             cx: &mut Context<Self>| {
                if let InputEvent::PressEnter { shift: false, .. } = event {
                    if let Some(idx) = this.editing_idx {
                        this.commit_edit(idx, window, cx);
                    }
                }
            },
        );
        let tag_name_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入分类名称..."));
        let timer = spawn_todo_timer(cx.weak_entity(), cx);

        Self {
            data,
            new_input,
            _new_input_sub: new_input_sub,
            editing_idx: None,
            edit_input,
            _edit_input_sub: edit_input_sub,
            tag_name_input,
            expanded_idx: None,
            show_completed: false,
            scroll_handle: ScrollHandle::new(),
            sidebar_scroll_handle: ScrollHandle::new(),
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

    fn finish_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_idx = None;
        self.edit_input.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        window.blur(cx);
    }

    fn commit_edit(&mut self, idx: usize, window: &mut Window, cx: &mut Context<Self>) {
        let trimmed = self.edit_input.read(cx).value().trim().to_string();
        if !trimmed.is_empty() {
            if let Some(item) = self.data.items.get_mut(idx) {
                item.text = trimmed;
            }
        }
        self.finish_editing(window, cx);
        TodoModel::save(&self.data, cx);
        cx.notify();
    }

    fn open_tag_edit(&mut self, tag: &TodoTag, window: &mut Window, cx: &mut Context<Self>) {
        let name = tag.name.clone();
        self.tag_name_input.update(cx, |state, cx| {
            state.set_value(name, window, cx);
        });
        self.tag_modal = Some(TagModalState::new_edit(tag));
        cx.notify();
    }

    fn open_tag_create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tag_name_input.update(cx, |state, cx| {
            state.set_value("新分类", window, cx);
        });
        self.tag_modal = Some(TagModalState::new_create());
        cx.notify();
    }

    fn close_tag_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tag_modal = None;
        self.tag_name_input.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        window.blur(cx);
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target_fg = gpui::hsla(0.0, 0.0, 0.98, 1.0);
        let target_muted = gpui::hsla(0.0, 0.0, 0.65, 1.0);
        let colors = &gpui_component::Theme::global(cx).colors;
        if colors.foreground != target_fg || colors.muted_foreground != target_muted {
            let theme_mut = gpui_component::Theme::global_mut(cx);
            theme_mut.colors.foreground = target_fg;
            theme_mut.colors.muted_foreground = target_muted;
        }

        let editing_idx = self.editing_idx;
        let expanded_idx = self.expanded_idx;
        let active_tag_id = &self.data.active_tag_id;
        let tags = &self.data.tags;
        let active_tag_obj = tags.iter().find(|t| &t.id == active_tag_id).cloned();

        let mut pending_elements = Vec::new();
        let mut completed_elements = Vec::new();

        let callbacks = crate::ui::item_card::ItemCardCallbacks {
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
                if let Some(item) = this.data.items.get(idx) {
                    let current = item.text.clone();
                    this.edit_input.update(cx, |state, cx| {
                        state.set_value(current, window, cx);
                    });
                    this.editing_idx = Some(idx);
                    cx.notify();
                }
            }),
            on_confirm_edit: std::rc::Rc::new(|this: &mut Self, window, cx, idx| {
                this.commit_edit(idx, window, cx);
            }),
            on_delete_item: std::rc::Rc::new(|this: &mut Self, window, cx, idx| {
                if this.editing_idx == Some(idx) {
                    this.finish_editing(window, cx);
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
            on_reorder_item: std::rc::Rc::new(|this: &mut Self, window, cx, src_id, dst_id| {
                if this.data.reorder_item(&src_id, &dst_id) {
                    if this.editing_idx.is_some() {
                        this.finish_editing(window, cx);
                    }
                    this.expanded_idx = None;
                    TodoModel::save(&this.data, cx);
                    cx.notify();
                }
            }),
        };

        for (idx, item) in self.data.items.iter().enumerate() {
            if active_tag_id != "all" && &item.tag_id != active_tag_id {
                continue;
            }

            let elem = render_todo_item(
                ItemCardProps {
                    idx,
                    item,
                    tags,
                    reminder_presets: &self.data.reminder_presets,
                    active_tag_id,
                    is_editing: editing_idx == Some(idx),
                    is_expanded: expanded_idx == Some(idx),
                    edit_input: &self.edit_input,
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
            .items_stretch()
            .size_full()
            .min_h_0()
            .gap(px(2.0))
            .overflow_hidden()
            // ── 左侧：吸附 Tab 侧边栏 ──────────────────────────────
            .child(render_sidebar(
                SidebarProps {
                    tags,
                    active_tag_id,
                    scroll_handle: &self.sidebar_scroll_handle,
                },
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
                |this, _, cx, from_idx, to_idx| {
                    this.data.reorder_tag(from_idx, to_idx);
                    TodoModel::save(&this.data, cx);
                    cx.notify();
                },
                cx,
            ))
            // ── 右侧：主体内容毛玻璃主面板 ───────────────────────────────
            .child(render_content_panel(
                ContentPanelProps {
                    active_tag_obj,
                    pending_count: pending_elements.len(),
                    can_delete_tag,
                    new_input: &self.new_input,
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
                    TagModalProps {
                        modal,
                        name_input: &self.tag_name_input,
                        can_delete: can_delete_tag,
                    },
                    |this, _, cx, color_idx| {
                        if let Some(m) = &mut this.tag_modal {
                            m.selected_color = color_idx;
                            cx.notify();
                        }
                    },
                    |this, window, cx, name, color_idx| {
                        if let Some(m) = &this.tag_modal {
                            let mut created = false;
                            match &m.mode {
                                TagModalMode::Create => {
                                    let new_id = this.data.add_tag(name, color_idx);
                                    this.data.active_tag_id = new_id;
                                    created = true;
                                }
                                TagModalMode::Edit { tag_id } => {
                                    this.data.update_tag(tag_id, name, color_idx);
                                }
                            }
                            TodoModel::save(&this.data, cx);
                            if created {
                                this.sidebar_scroll_handle.scroll_to_bottom();
                            }
                        }
                        this.close_tag_modal(window, cx);
                    },
                    |this, window, cx, tag_id| {
                        this.data.delete_tag_and_migrate(&tag_id);
                        TodoModel::save(&this.data, cx);
                        this.close_tag_modal(window, cx);
                    },
                    |this, window, cx| {
                        this.close_tag_modal(window, cx);
                    },
                    cx,
                ))
            })
    }
}
