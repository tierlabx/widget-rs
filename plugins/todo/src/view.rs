use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Icon, IconName};

use crate::model::{TodoItem, TodoModel};

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
    /// 顶部常驻新增输入框
    new_input: Entity<InputState>,
    /// 是否标记需要重置输入框（在 render 中判断）
    pending_reset: bool,
    /// 正在编辑的条目索引 + 对应输入框
    editing_idx: Option<usize>,
    edit_input: Entity<InputState>,
    show_completed: bool,
    scroll_handle: ScrollHandle,
}

impl TodoWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let saved_items = TodoModel::load(cx);

        // ── 顶部常驻输入框 ──────────────────────────────────────────
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
                        });
                        TodoModel::save(&this.items, cx);
                        this.scroll_handle.scroll_to_bottom();
                    }
                    // 标记需要在下次 render 时重置输入框 Entity
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
        // 确保在待办窗口中，Theme 文本颜色为纯白色
        gpui_component::Theme::global_mut(cx).colors.foreground = gpui::hsla(0.0, 0.0, 0.98, 1.0);
        gpui_component::Theme::global_mut(cx)
            .colors
            .muted_foreground = gpui::hsla(0.0, 0.0, 0.65, 1.0);

        // 如果收到了重置信号，在 render 入口处重建输入框 Entity（此时 window 可用）
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
        let edit_input = &self.edit_input;
        let new_input = &self.new_input;

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
                // ── 编辑状态：悬浮输入卡片 ──────────────────────────
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .gap(px(8.0))
                    .px(px(10.0))
                    .py(px(8.0))
                    .bg(rgba(0x0f172ae6)) // 高对比度深色悬浮胶囊
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(rgba(0x00d99266))
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .flex_shrink_0()
                            .rounded_full()
                            .border_2()
                            .border_color(rgb(0x00d992)),
                    )
                    .child(div().flex_1().child(Input::new(edit_input)))
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .bg(rgba(0x00d99225))
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
                // ── 悬浮条目胶囊卡片（独立悬浮于透明桌面）──
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
                    .py(px(9.0))
                    .gap(px(10.0))
                    .bg(if done {
                        rgba(0x0f172a75) // 已完成半透明淡深色
                    } else {
                        rgba(0x0f172ad0) // 未完成高质感深海蓝黑胶囊
                    })
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(if done {
                        rgba(0xffffff0a)
                    } else {
                        rgba(0xffffff18)
                    })
                    .hover(|s| s.bg(rgba(0x1e293be0)).border_color(rgba(0xffffff30)))
                    // 精致圆形勾选圈
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
                    // 待办文字（高清晰度纯白）
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
                            .child(text),
                    )
                    // 右侧动作按钮区
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(rgba(0xffffff88))
                                    .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0x00d992)))
                                    .id(ElementId::Name(format!("todo-edit-{idx}").into()))
                                    .on_click(edit_handler)
                                    .child(Icon::new(IconName::Redo).size(px(11.0))),
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
                                    .text_color(rgba(0xffffff88))
                                    .hover(|s| s.bg(rgba(0xff4d4d25)).text_color(rgb(0xff6b6b)))
                                    .id(ElementId::Name(format!("todo-del-{idx}").into()))
                                    .on_click(delete_handler)
                                    .child(Icon::new(IconName::Delete).size(px(11.0))),
                            ),
                    )
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
            .bg(rgba(0x0a1220b5)) // ~71% 优雅深海蓝黑半透明，完美透出壁纸又有清晰的组件轮廓
            .rounded(px(14.0))
            .border_1()
            .border_color(rgba(0xffffff22)) // 细腻微光玻璃边框
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
                    .bg(rgba(0x00000040)) // 微弱深色输入背景
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
                            .border_color(rgba(0xffffff88))
                            .flex()
                            .justify_center()
                            .items_center()
                            .text_color(rgb(0xffffff))
                            .child(Icon::new(IconName::Plus).size(px(10.0))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(Input::new(new_input).appearance(false).bordered(false)),
                    ),
            )
            // 2. 待办条目列表（卡片间自然透出桌面壁纸）
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
                    // 已完成折叠标头
                    .when(!completed_elements.is_empty(), |d| {
                        let show = self.show_completed;
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .w_full()
                                .px(px(10.0))
                                .py(px(6.0))
                                .mt(px(4.0))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .bg(rgba(0x0f172a60))
                                .border_1()
                                .border_color(rgba(0xffffff10))
                                .id("toggle-completed")
                                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                    this.show_completed = !this.show_completed;
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgba(0xffffff88))
                                        .child(format!("已完成 ({})", completed_elements.len())),
                                )
                                .child(
                                    div().text_color(rgba(0xffffff88)).child(
                                        Icon::new(if show {
                                            IconName::ChevronDown
                                        } else {
                                            IconName::ChevronRight
                                        })
                                        .size(px(12.0)),
                                    ),
                                ),
                        )
                    })
                    .when(self.show_completed && !completed_elements.is_empty(), |d| {
                        d.children(completed_elements)
                    }),
            )
    }
}
