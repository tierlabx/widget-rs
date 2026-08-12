use crate::layout::page_header;
use gpui::*;

pub fn render_widgets_content(
    cx: &mut Context<crate::main_window::MainWindow>,
) -> Vec<gpui::AnyElement> {
    let plugin_list = cx
        .try_global::<widget_core::PluginList>()
        .map(|list| list.0.clone())
        .unwrap_or_default();

    vec![
        div()
            .flex()
            .justify_between()
            .items_center()
            .w_full()
            .child(page_header(
                "小部件库 (市场)",
                "发现并安装社区开发的桌面功能扩展",
            ))
            .into_any_element(),
        div()
            .flex()
            .w_full()
            .gap(px(16.0))
            .flex_wrap()
            .pb(px(24.0))
            .children(plugin_list.into_iter().map(|meta| {
                let is_loaded = cx
                    .try_global::<widget_core::UIState>()
                    .is_none_or(|s| s.is_plugin_loaded(meta.id));
                market_plugin_card(
                    meta.name,
                    meta.id,
                    meta.description,
                    meta.icon,
                    meta.version,
                    meta.author,
                    is_loaded,
                )
            }))
            .into_any_element(),
    ]
}

#[allow(clippy::too_many_arguments)]
fn market_plugin_card(
    name: &'static str,
    id_str: &'static str,
    desc: &'static str,
    icon: gpui_component::IconName,
    version: &'static str,
    author: &'static str,
    is_loaded: bool,
) -> impl IntoElement {
    use crate::components::button::{Button, ButtonVariant};
    div()
        .flex()
        .flex_col()
        .w(px(320.0))
        .p(px(20.0))
        .gap(px(16.0))
        .bg(rgb(0x101010))
        .border_1()
        .border_color(rgb(0x3d3a39))
        .rounded(px(8.0))
        .hover(|s| s.border_color(rgba(0x00d99280)))
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(12.0))
                .child(
                    div()
                        .w(px(40.0))
                        .h(px(40.0))
                        .rounded(px(8.0))
                        .bg(rgba(0xffffff0a))
                        .flex()
                        .justify_center()
                        .items_center()
                        .child(
                            div()
                                .text_color(rgb(0xb8b3b0))
                                .child(gpui_component::Icon::new(icon)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xf2f2f2))
                                .child(name),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x8b949e))
                                .child(format!("v{} · by {}", version, author)),
                        ),
                ),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0xb8b3b0))
                .h(px(40.0))
                .child(desc),
        )
        .child(
            div().flex().w_full().gap(px(8.0)).child(
                Button::new(id_str, if is_loaded { "已安装" } else { "获取" })
                    .variant(if is_loaded {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Default
                    })
                    .on_click(|_, _, _| {
                        // TODO: 实现安装逻辑
                    }),
            ),
        )
}
