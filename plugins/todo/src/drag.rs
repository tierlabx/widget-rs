use gpui::*;

/// 正在被拖动的待办条目载荷
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraggedTodoItem {
    /// 待办项唯一 ID
    pub item_id: String,
    /// 是否已完成
    pub is_done: bool,
}

/// 拖拽跟随预览组件
pub struct TodoDragPreview {
    pub text: String,
    pub color_hex: u32,
}

impl Render for TodoDragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(5.0))
            .px(px(8.0))
            .py(px(4.0))
            .bg(rgba(0x0f172ae8))
            .rounded(px(6.0))
            .border_1()
            .border_color(rgb(self.color_hex))
            .shadow_lg()
            .child(
                div()
                    .w(px(2.5))
                    .h(px(12.0))
                    .rounded_full()
                    .bg(rgb(self.color_hex)),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(rgb(0xf8fafc))
                    .max_w(px(180.0))
                    .truncate()
                    .child(self.text.clone()),
            )
    }
}

/// 渲染精致微型点阵拖拽抓手（两列三行极细微点）
pub fn render_drag_handle() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(7.0))
        .h(px(14.0))
        .flex_shrink_0()
        .cursor_pointer()
        .opacity(0.35)
        .hover(|s| s.opacity(0.95))
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(1.5))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(dot())
                        .child(dot())
                        .child(dot()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(dot())
                        .child(dot())
                        .child(dot()),
                ),
        )
}

fn dot() -> impl IntoElement {
    div().size(px(1.5)).rounded_full().bg(rgba(0xffffffaa))
}
