use gpui::*;

pub struct MainWindow;

impl MainWindow {
    pub fn new() -> Self {
        Self
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_visible = cx.try_global::<widget_core::UIState>().map_or(true, |s| s.is_visible);
        
        if !is_visible {
            return div().bg(rgba(0x00000000)).into_any_element();
        }

        div()
            .flex()
            .size_full()
            .bg(rgba(0x050507f2)) // Abyss Black transparent
            .child(
                // Sidebar
                div()
                    .flex()
                    .flex_col()
                    .w(px(280.0))
                    .h_full()
                    .bg(rgb(0x101010)) // Carbon Surface
                    .border_r_1()
                    .border_color(rgb(0x3d3a39)) // Warm Charcoal
                    .child(
                        // sidebarHeader
                        div()
                            .flex()
                            .items_center()
                            .w_full()
                            .px(px(16.0))
                            .py(px(24.0))
                            .gap(px(16.0))
                            .child(
                                // logoIcon
                                div()
                                    .w(px(40.0))
                                    .h(px(40.0))
                                    .rounded(px(8.0))
                                    .bg(rgb(0x101010))
                                    .border_1()
                                    .border_color(rgb(0x3d3a39))
                            )
                            .child(
                                // appTitle
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child("Widget RS")
                            )
                    )
                    .child(
                        // sidebarNav
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .p(px(8.0))
                            .gap(px(4.0))
                            .child(
                                // navItem1 (Active)
                                div()
                                    .flex()
                                    .items_center()
                                    .w_full()
                                    .px(px(16.0))
                                    .py(px(12.0))
                                    .gap(px(16.0))
                                    .rounded(px(6.0))
                                    .bg(rgba(0x00d9921a)) // Emerald Signal transparent
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x00d992))
                                            .child("控制台核心 (Main Console)")
                                    )
                            )
                            .child(
                                // navItem2
                                div()
                                    .flex()
                                    .items_center()
                                    .w_full()
                                    .px(px(16.0))
                                    .py(px(12.0))
                                    .gap(px(16.0))
                                    .rounded(px(6.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xb8b3b0))
                                            .child("组件插件库")
                                    )
                            )
                    )
                    .child(div().flex_1()) // sidebarSpacer
                    .child(
                        // sidebarFooter
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .p(px(16.0))
                            .child(
                                // statusFrame
                                div()
                                    .flex()
                                    .items_center()
                                    .w_full()
                                    .p(px(12.0))
                                    .gap(px(8.0))
                                    .bg(rgb(0x050507))
                                    .rounded(px(6.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xb8b3b0))
                                            .child("系统运行正常")
                                    )
                            )
                    )
            )
            .child(
                // Main Content
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .p(px(24.0))
                    .gap(px(24.0))
                    .child(
                        // headerSection
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .w_full()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_2xl()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xf2f2f2))
                                            .child("欢迎回到 Widget RS")
                                    )
                            )
                    )
                    .child(
                        // widgetsSection
                        div()
                            .flex()
                            .flex_col()
                            .h_full()
                            .w_full()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .w_full()
                                    .child(
                                        div()
                                            .text_lg()
                                            .text_color(rgb(0xf2f2f2))
                                            .child("已加载的独立窗口部件")
                                    )
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .h_full()
                                    .w_full()
                                    .gap(px(16.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xb8b3b0))
                                            .child("便签小部件和待办事项小部件已在独立窗口运行。")
                                    )
                            )
                    )
            )
            .into_any_element()
    }
}
