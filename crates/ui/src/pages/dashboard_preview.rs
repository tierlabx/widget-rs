use gpui::*;

/// 渲染微缩预览视图
pub fn render_preview(plugin_id: &str) -> impl IntoElement {
    match plugin_id {
        "sticky_widget" => div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p(px(12.0))
            .rounded(px(6.0))
            .bg(rgb(0x050507))
            .border_1()
            .border_color(rgba(0x3d3a3940))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .w_full()
                    .p(px(8.0))
                    .bg(rgb(0x18181b))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xf2f2f2))
                            .font_weight(FontWeight::BOLD)
                            .child("桌面便签"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x8b949e))
                            .child("支持 Markdown 语法与多种便签主题..."),
                    ),
            ),
        "todo_widget" => div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .w_full()
            .h_full()
            .p(px(12.0))
            .rounded(px(6.0))
            .bg(rgb(0x050507))
            .border_1()
            .border_color(rgba(0x3d3a3940))
            .children(
                [
                    ("完成季度代码评审", true),
                    ("修复控制面板布局 Bug", false),
                    ("测试桌面常驻特性", false),
                ]
                .iter()
                .map(|(text, done)| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .w(px(12.0))
                                .h(px(12.0))
                                .rounded(px(3.0))
                                .border_1()
                                .border_color(if *done { rgb(0x00d992) } else { rgb(0x3f3f46) })
                                .bg(if *done {
                                    rgb(0x00d992)
                                } else {
                                    rgba(0x00000000)
                                })
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(if *done {
                                    div().text_color(rgb(0x050507)).child(
                                        gpui_component::Icon::new(gpui_component::IconName::Check)
                                            .size(px(8.0)),
                                    )
                                } else {
                                    div()
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if *done { rgb(0x71717a) } else { rgb(0xf2f2f2) })
                                .child(*text),
                        )
                }),
            ),
        "stretchly_widget" => div()
            .flex()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .p(px(12.0))
            .rounded(px(6.0))
            .bg(rgb(0x050507))
            .border_1()
            .border_color(rgba(0x3d3a3940))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .h_full()
                    .bg(rgba(0x00d99210))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(rgba(0x00d99240))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x00d992))
                            .font_weight(FontWeight::BOLD)
                            .child("休息提醒 - 专注中..."),
                    ),
            ),
        "fences_widget" => div()
            .flex()
            .flex_wrap()
            .content_start()
            .gap(px(6.0))
            .w_full()
            .h_full()
            .p(px(10.0))
            .rounded(px(6.0))
            .bg(rgb(0x050507))
            .border_1()
            .border_color(rgba(0x3d3a3940))
            .children(["代码", "文档", "工具", "杂项"].iter().map(|label| {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(3.0))
                    .p(px(6.0))
                    .w(px(48.0))
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .rounded(px(6.0))
                            .bg(rgba(0x60a5fa18))
                            .flex()
                            .justify_center()
                            .items_center()
                            .text_color(rgb(0x60a5fa))
                            .child(
                                gpui_component::Icon::new(gpui_component::IconName::Folder)
                                    .size(px(14.0)),
                            ),
                    )
                    .child(div().text_xs().text_color(rgb(0x8b949e)).child(*label))
            })),
        _ => div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .p(px(12.0))
            .rounded(px(6.0))
            .bg(rgb(0x050507))
            .border_1()
            .border_color(rgba(0x3d3a3940))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x8b949e))
                    .child("扩展桌面小组件"),
            ),
    }
}
