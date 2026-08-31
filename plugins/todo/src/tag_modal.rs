use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};

use crate::model::{TodoTag, GANTT_COLORS};

/// 标签编辑/新建弹窗模式
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagModalMode {
    Create,
    Edit { tag_id: String },
}

/// 标签编辑弹窗状态
pub struct TagModalState {
    pub mode: TagModalMode,
    pub name_input: Entity<InputState>,
    pub selected_color: usize,
}

impl TagModalState {
    pub fn new_create<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> Self {
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("新分类".to_string())
                .placeholder("输入分类名称...")
        });
        Self {
            mode: TagModalMode::Create,
            name_input,
            selected_color: 0,
        }
    }

    pub fn new_edit<V: 'static>(tag: &TodoTag, window: &mut Window, cx: &mut Context<V>) -> Self {
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(tag.name.clone())
                .placeholder("输入分类名称...")
        });
        Self {
            mode: TagModalMode::Edit {
                tag_id: tag.id.clone(),
            },
            name_input,
            selected_color: tag.gantt_color % GANTT_COLORS.len(),
        }
    }
}

/// 渲染标签编辑/新建浮层卡片
pub fn render_tag_modal<V: 'static>(
    modal: &TagModalState,
    can_delete: bool,
    on_select_color: impl Fn(&mut V, &mut Window, &mut Context<V>, usize) + 'static + Clone,
    on_save: impl Fn(&mut V, &mut Window, &mut Context<V>, String, usize) + 'static + Clone,
    on_delete: impl Fn(&mut V, &mut Window, &mut Context<V>, String) + 'static + Clone,
    on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let mode = modal.mode.clone();
    let name_input = &modal.name_input;
    let selected_color = modal.selected_color;
    let is_edit = matches!(mode, TagModalMode::Edit { .. });

    let on_close_cancel = on_close.clone();
    let on_close_bg = on_close.clone();

    let on_save_click = {
        let on_save = on_save.clone();
        let name_input = name_input.clone();
        cx.listener(move |this, _: &ClickEvent, window, cx| {
            let name = name_input.read(cx).value().to_string();
            let trimmed = name.trim().to_string();
            let final_name = if trimmed.is_empty() {
                "未命名分类".to_string()
            } else {
                trimmed
            };
            on_save(this, window, cx, final_name, selected_color);
        })
    };

    let on_delete_click = {
        let on_delete = on_delete.clone();
        let mode = mode.clone();
        cx.listener(move |this, _: &ClickEvent, window, cx| {
            if let TagModalMode::Edit { tag_id } = &mode {
                on_delete(this, window, cx, tag_id.clone());
            }
        })
    };

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        // 1. 半透明背景遮罩（点击关闭）
        .child(
            div()
                .absolute()
                .inset_0()
                .bg(rgba(0x00000088))
                .id("tag-modal-backdrop")
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    on_close_bg(this, window, cx);
                })),
        )
        // 2. 居中卡片内容（兄弟节点，点击卡片不会触发遮罩关闭）
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .w(px(260.0))
                .p(px(14.0))
                .gap(px(12.0))
                .bg(rgba(0x0f172af8))
                .rounded(px(12.0))
                .border_1()
                .border_color(rgba(0xffffff28))
                .shadow_lg()
                .id("tag-modal-card")
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                .on_click(|_, _, cx| cx.stop_propagation())
                // 1. 标题与关闭按钮
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xf8fafc))
                                .child(if is_edit {
                                    "编辑分类标签"
                                } else {
                                    "新建分类标签"
                                }),
                        )
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
                                .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0xffffff)))
                                .id("tag-modal-close-btn")
                                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                                    on_close_cancel(this, window, cx);
                                }))
                                .child(Icon::new(IconName::Close).size(px(10.0))),
                        ),
                )
                // 2. 名称输入
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgba(0xffffff80))
                                .child("分类名称"),
                        )
                        .child(
                            div()
                                .w_full()
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(rgba(0x00000040))
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(rgba(0xffffff18))
                                .child(Input::new(name_input).appearance(false).bordered(false)),
                        ),
                )
                // 3. 颜色选择器
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgba(0xffffff80))
                                .child("标记色系"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .w_full()
                                .children(GANTT_COLORS.iter().enumerate().map(|(c_idx, c_obj)| {
                                    let is_selected = c_idx == selected_color;
                                    let on_select = on_select_color.clone();
                                    div()
                                        .w(px(22.0))
                                        .h(px(22.0))
                                        .rounded_full()
                                        .cursor_pointer()
                                        .bg(rgb(c_obj.hex))
                                        .border_2()
                                        .border_color(if is_selected {
                                            rgb(0xffffff)
                                        } else {
                                            rgba(0x00000000)
                                        })
                                        .hover(|s| s.border_color(rgb(0xffffff)))
                                        .id(ElementId::Name(
                                            format!("tag-color-select-{c_idx}").into(),
                                        ))
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            on_select(this, window, cx, c_idx);
                                        }))
                                        .child(
                                            div()
                                                .size_full()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .when(is_selected, |d: Div| {
                                                    d.child(
                                                        Icon::new(IconName::Check)
                                                            .size(px(11.0))
                                                            .text_color(rgb(0xffffff)),
                                                    )
                                                }),
                                        )
                                })),
                        ),
                )
                // 4. 底部按钮区
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .pt(px(4.0))
                        .child(div().when(is_edit && can_delete, |d: Div| {
                            d.child(
                                div()
                                    .px(px(8.0))
                                    .py(px(4.0))
                                    .rounded(px(5.0))
                                    .cursor_pointer()
                                    .text_xs()
                                    .text_color(rgb(0xf87171))
                                    .bg(rgba(0xf8717120))
                                    .hover(|s| s.bg(rgba(0xf8717140)))
                                    .id("tag-modal-delete-btn")
                                    .on_click(on_delete_click)
                                    .child("删除分类"),
                            )
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .px(px(10.0))
                                        .py(px(4.0))
                                        .rounded(px(5.0))
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(rgba(0xffffff80))
                                        .bg(rgba(0xffffff10))
                                        .hover(|s| s.bg(rgba(0xffffff20)))
                                        .id("tag-modal-cancel-btn")
                                        .on_click(cx.listener(
                                            move |this, _: &ClickEvent, window, cx| {
                                                on_close(this, window, cx);
                                            },
                                        ))
                                        .child("取消"),
                                )
                                .child(
                                    div()
                                        .px(px(12.0))
                                        .py(px(4.0))
                                        .rounded(px(5.0))
                                        .cursor_pointer()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xffffff))
                                        .bg(rgb(0x38bdf8))
                                        .hover(|s| s.bg(rgb(0x0284c7)))
                                        .id("tag-modal-save-btn")
                                        .on_click(on_save_click)
                                        .child("保存"),
                                ),
                        ),
                ),
        )
}
