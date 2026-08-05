use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};

use gpui_component::{Icon, IconName};
use raw_window_handle::HasWindowHandle;

use crate::model::{TodoItem, TodoModel};

#[derive(Clone)]
struct DragTodo(usize);

struct DragTodoView {
    text: String,
}

impl Render for DragTodoView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0x3d3a39))
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
    hwnd_reported: bool,
    items: Vec<TodoItem>,
    /// 新增输入框
    new_input: Entity<InputState>,
    show_input: bool,
    /// 正在编辑的条目索引 + 对应输入框
    editing_idx: Option<usize>,
    edit_input: Entity<InputState>,
    show_completed: bool,
    scroll_handle: ScrollHandle,
}

impl TodoWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 从全局配置加载已持久化的待办数据
        let saved_items = TodoModel::load(cx);

        // ── 新增输入框 ──────────────────────────────────────────────
        let new_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入新待办，按 Enter 确认..."));
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
                        });
                        TodoModel::save(&this.items, cx);
                        this.scroll_handle.scroll_to_bottom();
                    }
                    this.show_input = false;
                    cx.notify();
                }
            },
        )
        .detach();

        // ── 编辑输入框（占位，实际每次编辑时重建）─────────────────
        let edit_input = cx.new(|cx| InputState::new(window, cx).placeholder("编辑待办内容..."));

        Self {
            hwnd_reported: false,
            items: saved_items,
            new_input,
            show_input: false,
            editing_idx: None,
            edit_input,
            show_completed: false,
            scroll_handle: ScrollHandle::new(),
        }
    }
}

impl Render for TodoWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit_mode = cx
            .try_global::<widget_core::UIState>()
            .is_some_and(|s| s.is_edit_mode);

        // ── Win32 HWND / 边框逻辑 ─────────────────────────────────
        if let Ok(handle) = _window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
                let hwnd = h.hwnd.get();
                if !self.hwnd_reported {
                    self.hwnd_reported = true;
                    let _ = hwnd;
                }
            }
        }
        widget_core::update_window_edit_mode(_window, is_edit_mode);

        // ── 编辑模式拖拽条 ────────────────────────────────────────
        let drag_handle = if is_edit_mode {
            Some(
                div()
                    .w_full()
                    .h(px(28.0))
                    .bg(rgb(0x00d992))
                    .flex()
                    .justify_center()
                    .items_center()
                    .id("todo-drag")
                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                        widget_core::start_window_drag(window);
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x050507))
                            .child(":: 拖拽移动待办 ::"),
                    ),
            )
        } else {
            None
        };

        let done_count = self.items.iter().filter(|i| i.done).count();
        let total = self.items.len();
        let editing_idx = self.editing_idx;
        let edit_input = &self.edit_input;
        let new_input = &self.new_input;
        let show_input = self.show_input;

        // ── 条目列表 ──────────────────────────────────────────────
        let mut pending_elements = Vec::new();
        let mut completed_elements = Vec::new();

        for (idx, item) in self.items.iter().enumerate() {
            let done = item.done;
            let text = item.text.clone();
            let is_editing = editing_idx == Some(idx);

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
                if idx < this.items.len() {
                    this.items.remove(idx);
                }
                TodoModel::save(&this.items, cx);
                cx.notify();
            });

            let edit_handler = cx.listener(move |this, _: &ClickEvent, window, cx| {
                if this.editing_idx == Some(idx) {
                    // 已在编辑此条：提交
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
                    // 重建 InputState 并用 default_value 预填当前文字
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
                    this.show_input = false;
                }
                cx.notify();
            });

            let item_element = if is_editing {
                // ── 编辑状态：显示输入框 ──────────────────────────
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .gap(px(8.0))
                    .px(px(10.0))
                    .py(px(4.0))
                    // 编辑行容器：整行使用 Input 默认外观（白底黑字）
                    .child(
                        // 勾选圆圈（灰色，编辑中不可点击）
                        div()
                            .w(px(22.0))
                            .h(px(22.0))
                            .flex_shrink_0()
                            .rounded_full()
                            .border_2()
                            .border_color(rgb(0x4a4a4e))
                            .bg(rgba(0x00000000)),
                    )
                    .child(
                        // Input 保持默认 appearance，让它自带白色背景+深色文字
                        div().flex_1().child(Input::new(edit_input)),
                    )
                    // 确认按钮
                    .child(
                        div()
                            .w(px(28.0))
                            .h(px(28.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .bg(rgba(0x00d99220))
                            .text_color(rgb(0x00d992))
                            .hover(|s| s.bg(rgba(0x00d99240)))
                            .id(ElementId::Name(format!("todo-confirm-{idx}").into()))
                            .on_click(edit_handler)
                            .child(Icon::new(IconName::Check).size(px(13.0))),
                    )
                    // 删除按钮
                    .child(
                        div()
                            .w(px(28.0))
                            .h(px(28.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .bg(rgba(0xff4d4d15))
                            .text_color(rgb(0xff6b6b))
                            .hover(|s| s.bg(rgba(0xff4d4d30)))
                            .id(ElementId::Name(format!("todo-del-{idx}").into()))
                            .on_click(delete_handler)
                            .child(Icon::new(IconName::Delete).size(px(13.0))),
                    )
                    .into_any_element()
            } else {
                // ── 普通显示状态 ──────────────────────────────────
                let drag_text = text.clone();
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
                    .items_center()
                    .w_full()
                    .px(px(12.0))
                    .py(px(10.0))
                    .gap(px(12.0))
                    .bg(rgb(0x101010)) // VoltAgent Carbon Surface
                    .border_1()
                    .border_color(rgb(0x3d3a39)) // VoltAgent Warm Charcoal
                    .rounded(px(6.0))
                    .hover(|s| s.border_color(rgba(0x00d99240)))
                    // 勾选圆圈
                    .child(
                        div()
                            .w(px(22.0))
                            .h(px(22.0))
                            .flex_shrink_0()
                            .rounded_full()
                            .border_2()
                            .cursor_pointer()
                            .id(ElementId::Name(format!("todo-check-{idx}").into()))
                            .border_color(if done { rgb(0x00d992) } else { rgb(0x4a4a4e) })
                            .bg(if done {
                                rgba(0x00d99230)
                            } else {
                                rgba(0x00000000)
                            })
                            .flex()
                            .justify_center()
                            .items_center()
                            .on_click(toggle_handler)
                            .when(done, |d: Stateful<Div>| {
                                d.child(
                                    div()
                                        .text_color(rgb(0x00d992))
                                        .child(Icon::new(IconName::Check).size(px(12.0))),
                                )
                            }),
                    )
                    // 待办文字（点击进入编辑）
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(if done { rgb(0x6e6e7a) } else { rgb(0xe8e8ea) })
                            .when(done, |d: Div| d.line_through())
                            .child(text),
                    )
                    // 编辑按钮
                    .child(
                        div()
                            .w(px(26.0))
                            .h(px(26.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_color(rgb(0x5a5a64))
                            .hover(|s| s.bg(rgba(0xffffff10)).text_color(rgb(0x00d992)))
                            .id(ElementId::Name(format!("todo-edit-{idx}").into()))
                            .on_click(edit_handler)
                            .child(Icon::new(IconName::Redo).size(px(12.0))),
                    )
                    // 删除按钮
                    .child(
                        div()
                            .w(px(26.0))
                            .h(px(26.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_color(rgb(0x5a5a64))
                            .hover(|s| s.bg(rgba(0xff4d4d20)).text_color(rgb(0xff6b6b)))
                            .id(ElementId::Name(format!("todo-del-{idx}").into()))
                            .on_click(delete_handler)
                            .child(Icon::new(IconName::Delete).size(px(12.0))),
                    )
                    .into_any_element()
            };

            if done {
                completed_elements.push(item_element);
            } else {
                pending_elements.push(item_element);
            }
        }

        // ── 整体布局 ──────────────────────────────────────────────
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgba(0x050507f2)) // VoltAgent Abyss Black with slight transparency
            .border_1()
            .border_color(if is_edit_mode {
                rgb(0x00d992)
            } else {
                rgb(0x3d3a39) // VoltAgent Warm Charcoal
            })
            .rounded(px(8.0))
            .children(drag_handle)
            // 标题栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px(px(16.0))
                    .bg(rgb(0x050507)) // VoltAgent Abyss Black
                    .border_b_1()
                    .border_color(rgb(0x3d3a39)) // VoltAgent Warm Charcoal
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_color(rgb(0x00d992)) // VoltAgent Emerald Green
                                    .child(Icon::new(IconName::CircleCheck).size(px(16.0))),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xf0f0f2))
                                    .child("待办事项"),
                            ),
                    )
                    .child(
                        div()
                            .px(px(8.0))
                            .py(px(2.0))
                            .rounded_full()
                            .bg(rgba(0x00d99220))
                            .text_xs()
                            .text_color(rgb(0x00d992))
                            .child(format!("{}/{}", done_count, total)),
                    ),
            )
            // 条目列表
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .w_full()
                    .p(px(10.0))
                    .gap(px(5.0))
                    .id("todo-list-scroll")
                    .track_scroll(&self.scroll_handle)
                    .overflow_y_scroll()
                    .children(pending_elements)
                    .when(!completed_elements.is_empty(), |d| {
                        let show = self.show_completed;
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .w_full()
                                .mt(px(10.0))
                                .mb(px(4.0))
                                .cursor_pointer()
                                .id("toggle-completed")
                                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                    this.show_completed = !this.show_completed;
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x6e6e7a))
                                        .child(format!("已完成 ({})", completed_elements.len())),
                                )
                                .child(div().text_xs().text_color(rgb(0x6e6e7a)).child(if show {
                                    "▼"
                                } else {
                                    "▶"
                                })),
                        )
                    })
                    .when(self.show_completed && !completed_elements.is_empty(), |d| {
                        d.children(completed_elements)
                    })
                    // 新增输入框：使用默认 Input 外观（白底黑字，完全可见）
                    .when(show_input, |d| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .w_full()
                                .gap(px(8.0))
                                .px(px(10.0))
                                .py(px(4.0))
                                .child(
                                    div()
                                        .w(px(22.0))
                                        .h(px(22.0))
                                        .flex_shrink_0()
                                        .rounded_full()
                                        .border_2()
                                        .border_color(rgb(0x4a4a4e)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        // 不传 appearance(false)，保留默认白色背景+深色文字
                                        .child(Input::new(new_input)),
                                ),
                        )
                    }),
            )
            // 底部"添加"按钮
            .child(
                div()
                    .flex()
                    .justify_center()
                    .items_center()
                    .w_full()
                    .px(px(12.0))
                    .py(px(12.0))
                    .gap(px(6.0))
                    .border_t_1()
                    .border_color(rgb(0x3d3a39)) // VoltAgent Warm Charcoal
                    .bg(rgb(0x050507))
                    .cursor_pointer()
                    .id("todo-add-btn")
                    .hover(|s| s.bg(rgba(0x00d9921a)))
                    .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                        this.show_input = !this.show_input;
                        if this.show_input {
                            // 关闭正在编辑的条目
                            this.editing_idx = None;

                            // 重新初始化 InputState，清空之前的输入
                            let new_input = cx.new(|cx| {
                                InputState::new(window, cx)
                                    .placeholder("输入新待办，按 Enter 确认...")
                            });
                            cx.subscribe(
                                &new_input,
                                |this: &mut Self,
                                 input: Entity<InputState>,
                                 event: &InputEvent,
                                 cx| {
                                    if let InputEvent::PressEnter { .. } = event {
                                        let text = input.read(cx).value().to_string();
                                        let trimmed = text.trim().to_string();
                                        if !trimmed.is_empty() {
                                            this.items.push(TodoItem {
                                                text: trimmed,
                                                done: false,
                                            });
                                            TodoModel::save(&this.items, cx);
                                            this.scroll_handle.scroll_to_bottom();
                                        }
                                        this.show_input = false;
                                        cx.notify();
                                    }
                                },
                            )
                            .detach();
                            this.new_input = new_input;

                            // 打开输入框时也滚动到底部
                            this.scroll_handle.scroll_to_bottom();
                        }
                        cx.notify();
                    }))
                    .child(div().text_color(rgb(0x00d992)).child(if show_input {
                        Icon::new(IconName::Minus).size(px(14.0)).into_any_element()
                    } else {
                        Icon::new(IconName::Plus).size(px(14.0)).into_any_element()
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x00d992))
                            .font_weight(FontWeight::MEDIUM)
                            .child(if show_input { "取消" } else { "添加待办" }),
                    ),
            )
    }
}
