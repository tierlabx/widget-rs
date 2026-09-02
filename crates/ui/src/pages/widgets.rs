use gpui::*;
use std::time::Duration;

use crate::layout::page_header;
use crate::pages::widgets_card::render_market_card;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum WidgetsFilter {
    #[default]
    All,
    Installed,
    Available,
}

pub fn render_widgets_content(
    filter: WidgetsFilter,
    anim_token: u32,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> Vec<gpui::AnyElement> {
    let plugin_list = cx
        .try_global::<widget_core::PluginList>()
        .map(|list| list.0.clone())
        .unwrap_or_default();

    let total_count = plugin_list.len();
    let plugins_with_status: Vec<_> = plugin_list
        .into_iter()
        .map(|meta| {
            let is_loaded = cx
                .try_global::<widget_core::UIState>()
                .is_none_or(|s| s.is_plugin_loaded(meta.id));
            (meta, is_loaded)
        })
        .collect();

    let installed_count = plugins_with_status
        .iter()
        .filter(|(_, loaded)| *loaded)
        .count();
    let available_count = total_count.saturating_sub(installed_count);

    let filtered_plugins: Vec<_> = plugins_with_status
        .into_iter()
        .filter(|(_, loaded)| match filter {
            WidgetsFilter::All => true,
            WidgetsFilter::Installed => *loaded,
            WidgetsFilter::Available => !*loaded,
        })
        .collect();

    vec![
        // 顶部标题与分类过滤器
        div()
            .flex()
            .justify_between()
            .items_end()
            .w_full()
            .child(page_header(
                "小部件库 (市场)",
                "发现并安装社区与官方开发的桌面功能扩展，打造专属工作台",
            ))
            .child(render_filter_tabs(
                filter,
                total_count,
                installed_count,
                available_count,
                cx,
            ))
            .into_any_element(),
        // 插件网格列表或空状态
        if filtered_plugins.is_empty() {
            render_empty_state(filter, anim_token).into_any_element()
        } else {
            div()
                .flex()
                .w_full()
                .gap(px(16.0))
                .flex_wrap()
                .pb(px(24.0))
                .children(filtered_plugins.into_iter().enumerate().map(
                    |(idx, (meta, is_loaded))| {
                        render_market_card(
                            meta.name,
                            meta.id,
                            meta.description,
                            meta.icon,
                            meta.version,
                            meta.author,
                            meta.estimated_memory,
                            is_loaded,
                            idx,
                            anim_token,
                        )
                    },
                ))
                .into_any_element()
        },
    ]
}

fn render_filter_tabs(
    current: WidgetsFilter,
    total: usize,
    installed: usize,
    available: usize,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(4.0))
        .p(px(4.0))
        .rounded(px(8.0))
        .bg(rgb(0x101014))
        .border_1()
        .border_color(rgb(0x27272a))
        .child(filter_tab_button(
            "filter-tab-all",
            "全部",
            total,
            current == WidgetsFilter::All,
            WidgetsFilter::All,
            cx,
        ))
        .child(filter_tab_button(
            "filter-tab-installed",
            "已安装",
            installed,
            current == WidgetsFilter::Installed,
            WidgetsFilter::Installed,
            cx,
        ))
        .child(filter_tab_button(
            "filter-tab-available",
            "待发现",
            available,
            current == WidgetsFilter::Available,
            WidgetsFilter::Available,
            cx,
        ))
}

fn filter_tab_button(
    id_str: &'static str,
    label: &'static str,
    count: usize,
    is_active: bool,
    target_filter: WidgetsFilter,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> impl IntoElement {
    let handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
        if this.widgets_filter != target_filter {
            this.widgets_filter = target_filter;
            this.widgets_anim_token = this.widgets_anim_token.wrapping_add(1);
            cx.notify();
        }
    });

    let base = div()
        .id(id_str)
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(12.0))
        .py(px(6.0))
        .rounded(px(6.0))
        .cursor_pointer()
        .on_click(handler);

    let styled = if is_active {
        base.bg(rgba(0x00d9921a))
            .border_1()
            .border_color(rgba(0x00d99260))
            .text_color(rgb(0x00d992))
    } else {
        base.bg(rgba(0x00000000))
            .border_1()
            .border_color(rgba(0x00000000))
            .text_color(rgb(0x8b949e))
            .hover(|s| s.bg(rgba(0xffffff0a)).text_color(rgb(0xf2f2f2)))
    };

    styled
        .child(
            div()
                .text_sm()
                .font_weight(if is_active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .child(label),
        )
        .child(
            div()
                .px(px(6.0))
                .py(px(1.0))
                .rounded_full()
                .bg(if is_active {
                    rgba(0x00d9922b)
                } else {
                    rgba(0xffffff0f)
                })
                .text_xs()
                .child(count.to_string()),
        )
}

fn render_empty_state(filter: WidgetsFilter, anim_token: u32) -> impl IntoElement {
    let msg = match filter {
        WidgetsFilter::Installed => "暂无已安装的小部件，可在「待发现」中一键获取",
        WidgetsFilter::Available => "所有小部件均已安装完毕！可在控制面板中进行排版",
        WidgetsFilter::All => "暂无可用的小部件",
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .w_full()
        .py(px(80.0))
        .gap(px(16.0))
        .rounded(px(12.0))
        .bg(rgba(0xffffff03))
        .border_1()
        .border_color(rgba(0xffffff08))
        .child(
            div()
                .w(px(56.0))
                .h(px(56.0))
                .rounded_full()
                .bg(rgba(0xffffff08))
                .flex()
                .justify_center()
                .items_center()
                .text_color(rgb(0x8b949e))
                .child(gpui_component::Icon::new(
                    gpui_component::IconName::LayoutDashboard,
                )),
        )
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x8b949e))
                .child(msg),
        )
        .with_animation(
            ElementId::Name(format!("market-empty-{}", anim_token).into()),
            Animation::new(Duration::from_millis(250)).with_easing(gpui::ease_in_out),
            |el, delta| {
                let progress = delta.clamp(0.0, 1.0);
                el.opacity(progress)
            },
        )
}
