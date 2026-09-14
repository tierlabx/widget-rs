use crate::main_window::NavPage;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub fn render_sidebar(
    window: &mut Window,
    nav_page: NavPage,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .w(px(210.0))
        .h_full()
        .bg(rgb(0x101010))
        .border_r_1()
        .border_color(rgb(0x3d3a39))
        .pt(px(24.0))
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .px(px(8.0))
                .gap(px(4.0))
                .child(nav_item(
                    "控制面板",
                    gpui_component::IconName::WindowMaximize,
                    NavPage::Dashboard,
                    nav_page,
                    cx,
                ))
                .child(nav_item(
                    "小部件库",
                    gpui_component::IconName::LayoutDashboard,
                    NavPage::Widgets,
                    nav_page,
                    cx,
                ))
                .child(nav_item(
                    "设置",
                    gpui_component::IconName::Settings,
                    NavPage::Settings,
                    nav_page,
                    cx,
                )),
        )
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .p(px(16.0))
                .pb(px(32.0))
                .child(
                    div()
                        .relative()
                        .w_full()
                        .h(px(150.0))
                        .child(gpui_fps::fps_monitor(window, cx)),
                ),
        )
}

fn nav_item(
    label: &'static str,
    icon: gpui_component::IconName,
    page: NavPage,
    current: NavPage,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> impl IntoElement {
    let active = page == current;
    let handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
        this.nav_page = page;
        cx.notify();
    });
    div()
        .flex()
        .items_center()
        .w_full()
        .px(px(16.0))
        .py(px(10.0))
        .gap(px(16.0))
        .rounded(px(8.0))
        .when(active, |d: gpui::Div| {
            d.bg(rgba(0x00d9921a))
                .border_1()
                .border_color(rgba(0x00d99220))
        })
        .id(ElementId::Name(label.into()))
        .cursor_pointer()
        .hover(
            move |s: gpui::StyleRefinement| {
                if !active {
                    s.bg(rgba(0xffffff08))
                } else {
                    s
                }
            },
        )
        .on_click(handler)
        .child(
            div()
                .text_color(if active { rgb(0x00d992) } else { rgb(0x8b949e) })
                .child(gpui_component::Icon::new(icon)),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if active { rgb(0xf2f2f2) } else { rgb(0xb8b3b0) })
                .child(label),
        )
}
