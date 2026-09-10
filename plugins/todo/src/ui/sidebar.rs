use gpui::*;

use crate::model::{TodoTag, GANTT_COLORS};

/// 标签拖拽排序载荷
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraggedTag {
    pub from_idx: usize,
}

/// 标签拖拽跟随预览
struct TagDragPreview {
    text: String,
    color_hex: u32,
}

impl Render for TagDragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .px(px(8.0))
            .py(px(4.0))
            .bg(rgba(0x0f172af0))
            .rounded(px(6.0))
            .border_1()
            .border_color(rgb(self.color_hex))
            .shadow_lg()
            .child(
                div()
                    .w(px(3.0))
                    .h(px(12.0))
                    .rounded_full()
                    .bg(rgb(self.color_hex)),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xf8fafc))
                    .child(self.text.clone()),
            )
    }
}

/// 渲染左侧吸附式分类标签侧边栏（宽度自动，高度固定，支持拖动排序）
pub fn render_sidebar<V: 'static>(
    tags: &[TodoTag],
    active_tag_id: &str,
    on_select_tag: impl Fn(&mut V, &mut Window, &mut Context<V>, String) + 'static + Clone,
    on_edit_tag: impl Fn(&mut V, &mut Window, &mut Context<V>, TodoTag) + 'static + Clone,
    on_add_tag_click: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_reorder_tag: impl Fn(&mut V, &mut Window, &mut Context<V>, usize, usize) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let active_tag_id = active_tag_id.to_string();

    div()
        .w(px(104.0)) // 104px 浮动空间基准，长标签向左延伸不撞窗口左边界，右边缘紧贴内容面板
        .flex()
        .flex_col()
        .items_end() // Tab 右对齐，紧贴内容面板左边缘，文字长时向左朝外超出
        .gap(px(3.0))
        .pt(px(16.0))
        .pb(px(8.0))
        // 不加 overflow_hidden，允许长标签朝左边超出，不裁剪文字
        // 1. "全部" 分类 Tab（不可拖动）
        .child({
            let is_active = active_tag_id == "all";
            let on_select = on_select_tag.clone();
            div()
                .h(px(32.0))
                .rounded_l(px(8.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .justify_center()
                .whitespace_nowrap() // 防止换行
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
                // "全部" 也作为拖拽目标（拖到最前面）
                .drag_over::<DraggedTag>(|s, _, _, _| {
                    s.border_color(rgb(0x38bdf8)).bg(rgba(0x38bdf825))
                })
                .id("todo-tab-all")
                .on_click(cx.listener(move |this, _, window, cx| {
                    on_select(this, window, cx, "all".to_string());
                }))
                .child("全部")
        })
        // 2. 各自定义分类 Tab（可拖动排序）
        .children(tags.iter().enumerate().map(|(tag_idx, tag)| {
            let is_active = active_tag_id == tag.id;
            let tag_clone = tag.clone();
            let tag_id_clone = tag.id.clone();
            let tag_color = &GANTT_COLORS[tag.gantt_color % GANTT_COLORS.len()];
            let on_select = on_select_tag.clone();
            let on_edit = on_edit_tag.clone();
            let on_reorder = on_reorder_tag.clone();
            let contrast_text = tag_color.contrast_text();
            let preview_text = tag.name.clone();
            let preview_color = tag_color.hex;

            div()
                .relative()
                .h(px(32.0))
                .rounded_l(px(8.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(4.0))
                .whitespace_nowrap() // 防止换行
                .cursor_grab()
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
                // id 必须在 on_drag 之前，将 Div 转为 Stateful<Div>
                .id(ElementId::Name(format!("todo-tab-{}", tag.id).into()))
                // 拖拽悬停高亮
                .drag_over::<DraggedTag>(|s, _, _, _| {
                    s.border_color(rgb(0x38bdf8)).bg(rgba(0x38bdf820))
                })
                // 接受放下：执行排序
                .on_drop(cx.listener(move |this, drag: &DraggedTag, window, cx| {
                    on_reorder(this, window, cx, drag.from_idx, tag_idx);
                }))
                // 开始拖拽：生成预览
                .on_drag(DraggedTag { from_idx: tag_idx }, move |_, _, _, cx| {
                    cx.new(|_| TagDragPreview {
                        text: preview_text.clone(),
                        color_hex: preview_color,
                    })
                })
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
        // 3. 底部"+"新建分类按钮
        .child(
            div()
                .h(px(28.0))
                .rounded_l(px(6.0))
                .px(px(8.0))
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
                .child(gpui_component::Icon::new(gpui_component::IconName::Plus).size(px(12.0))),
        )
}
