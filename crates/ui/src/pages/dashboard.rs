use gpui::*;

use crate::layout::page_header;
use crate::pages::dashboard_stats::{get_private_memory_usage, render_stat_card};
use crate::pages::dashboard_widgets::render_widget_card;

/// 渲染控制面板主页面内容
pub fn render_dashboard_content(
    is_edit_mode: bool,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> Vec<gpui::AnyElement> {
    let plugin_list = cx
        .try_global::<widget_core::PluginList>()
        .map(|list| list.0.clone())
        .unwrap_or_default();

    let plugins_info: Vec<_> = plugin_list
        .iter()
        .map(|meta| {
            let loaded = cx
                .try_global::<widget_core::UIState>()
                .is_none_or(|s| s.is_plugin_loaded(meta.id));
            let enabled = cx
                .try_global::<widget_core::UIState>()
                .is_none_or(|s| s.is_plugin_enabled(meta.id));
            let top = cx
                .try_global::<widget_core::AppConfig>()
                .and_then(|c| c.plugins.get(meta.id))
                .is_some_and(|p| p.always_on_top);
            let pass = cx
                .try_global::<widget_core::AppConfig>()
                .and_then(|c| c.plugins.get(meta.id))
                .is_some_and(|p| p.mouse_passthrough);
            let estimated_memory = meta.estimated_memory;

            (meta.clone(), loaded, enabled, top, pass, estimated_memory)
        })
        .collect();

    let total_widgets = plugins_info.len();
    let running_widgets = plugins_info
        .iter()
        .filter(|(_, l, e, _, _, _)| *l && *e)
        .count();
    let stopped_widgets = total_widgets - running_widgets;

    let total_mem = get_private_memory_usage();
    let mem_str = format!("{:.1} MB", total_mem as f64 / 1024.0 / 1024.0);

    vec![
        div()
            .flex()
            .justify_between()
            .items_center()
            .w_full()
            .child(page_header("控制面板", "管理您的桌面小部件"))
            .child(
                div().flex().gap(px(16.0)).items_center().child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(8.0))
                        .px(px(20.0))
                        .py(px(12.0))
                        .rounded(px(8.0))
                        .bg(rgba(0x00d99218))
                        .border_1()
                        .border_color(rgb(0x00d992))
                        .id("edit-mode-btn")
                        .cursor_pointer()
                        .hover(|s| s.bg(rgba(0x00d99230)))
                        .on_click(|_, _, cx| {
                            let mut was_edit_mode = false;
                            let mut is_edit = false;
                            cx.update_global::<widget_core::UIState, _>(|s, _| {
                                was_edit_mode = s.is_edit_mode;
                                s.is_edit_mode = !s.is_edit_mode;
                                is_edit = s.is_edit_mode;
                            });
                            widget_core::NATIVE_EDIT_MODE
                                .store(is_edit, std::sync::atomic::Ordering::SeqCst);

                            if was_edit_mode {
                                if let Some(cb) = cx.try_global::<widget_core::SaveBoundsCallback>()
                                {
                                    let cb = cb.0.clone();
                                    cb(cx);
                                }
                            }
                            cx.refresh_windows();
                        })
                        .child(
                            div()
                                .text_base()
                                .text_color(rgb(0x2fd6a1))
                                .child(if is_edit_mode { "✓" } else { "+" }),
                        )
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x2fd6a1))
                                .child(if is_edit_mode {
                                    "完成排版"
                                } else {
                                    "添加 / 排版"
                                }),
                        ),
                ),
            )
            .into_any_element(),
        div()
            .flex()
            .w_full()
            .gap(px(12.0))
            .child(render_stat_card(
                gpui_component::IconName::Star,
                running_widgets.to_string(),
                "运行中",
                rgb(0x00d992),
                rgba(0x00d9920d),
                rgba(0x00d99225),
            ))
            .child(render_stat_card(
                gpui_component::IconName::CircleX,
                stopped_widgets.to_string(),
                "已停止",
                rgb(0x8b949e),
                rgba(0xffffff06),
                rgba(0x3d3a3960),
            ))
            .child(render_stat_card(
                gpui_component::IconName::GalleryVerticalEnd,
                total_widgets.to_string(),
                "小部件总数",
                rgb(0xb8b3b0),
                rgba(0xffffff06),
                rgba(0x3d3a3960),
            ))
            .child(render_stat_card(
                gpui_component::IconName::LayoutDashboard,
                mem_str,
                "主进程物理内存",
                rgb(0x00d992),
                rgba(0x00d9920d),
                rgba(0x00d99225),
            ))
            .into_any_element(),
        div()
            .flex()
            .flex_col()
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
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0xf2f2f2))
                            .child("已安装的小部件"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x8b949e))
                            .child(format!("{} 个小部件", total_widgets)),
                    ),
            )
            .child(div().flex().w_full().gap(px(16.0)).flex_wrap().children(
                plugins_info.into_iter().enumerate().filter_map(
                    |(i, (meta, loaded, enabled, top, pass, mem))| {
                        loaded.then(|| {
                            render_widget_card(
                                meta.name, meta.id, meta.icon, loaded, enabled, top, pass, i as u8,
                                mem,
                            )
                        })
                    },
                ),
            ))
            .into_any_element(),
    ]
}
