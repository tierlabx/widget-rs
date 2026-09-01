use gpui::*;

use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::Card;

/// 渲染控制面板已安装的小部件卡片（包含微缩预览、置顶、穿透、设置、加载、启用按钮）
#[allow(clippy::too_many_arguments)]
pub fn render_widget_card(
    title: &'static str,
    plugin_id: &'static str,
    icon: gpui_component::IconName,
    is_loaded: bool,
    is_enabled: bool,
    always_on_top: bool,
    mouse_passthrough: bool,
    has_settings: bool,
    kind: u8,
    estimated_memory: usize,
) -> impl IntoElement {
    let load_label: &'static str = if is_loaded { "卸载" } else { "加载" };
    let enable_label: &'static str = if is_enabled { "关闭" } else { "启用" };

    let status_badge = if !is_loaded {
        Badge::new("未加载")
            .variant(BadgeVariant::Outline)
            .show_dot(false)
    } else if is_enabled {
        Badge::new("运行中")
            .variant(BadgeVariant::Default)
            .show_dot(true)
    } else {
        Badge::new("已关闭")
            .variant(BadgeVariant::Secondary)
            .show_dot(true)
    };

    let preview = match plugin_id {
        "sticky_widget" => div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p(px(12.0))
            .rounded(px(6.0))
            .bg(rgb(0xfef3c7))
            .border_1()
            .border_color(rgba(0xf59e0b40))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x78350f))
                    .child("这是一个便签示例..."),
            ),
        "todo_widget" => div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p(px(12.0))
            .gap(px(6.0))
            .rounded(px(6.0))
            .bg(rgb(0x050507))
            .border_1()
            .border_color(rgba(0x3d3a3940))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .p(px(8.0))
                    .rounded(px(4.0))
                    .bg(rgba(0xffffff05))
                    .child(
                        div()
                            .w(px(12.0))
                            .h(px(12.0))
                            .rounded_full()
                            .bg(rgb(0x00d992)),
                    )
                    .child(div().text_sm().text_color(rgb(0x8b949e)).child("编写文档")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .p(px(8.0))
                    .rounded(px(4.0))
                    .bg(rgba(0xffffff05))
                    .child(
                        div()
                            .w(px(12.0))
                            .h(px(12.0))
                            .rounded_full()
                            .border_2()
                            .border_color(rgb(0x00d992)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xf2f2f2))
                            .child("完成项目设计"),
                    ),
            ),
        "stretchly_widget" => div()
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
            .w_full()
            .h_full()
            .p(px(8.0))
            .gap(px(6.0))
            .rounded(px(6.0))
            .bg(rgb(0x0d1117))
            .border_1()
            .border_color(rgba(0x3d3a3940))
            .children(["文档", "下载", "项目", "工具"].iter().map(|label| {
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
            .child(div().text_sm().text_color(rgb(0x8b949e)).child("暂无预览")),
    };

    Card::new()
        .fixed_height(px(210.0))
        .header(
            div()
                .flex()
                .justify_between()
                .items_center()
                .w_full()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .flex()
                                .justify_center()
                                .items_center()
                                .w(px(32.0))
                                .h(px(32.0))
                                .rounded(px(6.0))
                                .bg(rgba(0x00d9921a))
                                .border_1()
                                .border_color(rgba(0x00d99240))
                                .child(
                                    div()
                                        .text_color(rgb(0x00d992))
                                        .child(gpui_component::Icon::new(icon)),
                                    ),
                        )
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xf2f2f2))
                                .child(title),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x8b949e))
                                .child(format!("~{:.1} MB", estimated_memory as f64 / 1024.0 / 1024.0)),
                        ),
                )
                .child(status_badge),
        )
        .content(preview)
        .footer(
            div()
                .flex()
                .w_full()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(
                            div()
                                .id(SharedString::from(format!("{}-pin", plugin_id)))
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .p(px(6.0))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .bg(if always_on_top { rgba(0x00d99230) } else { rgba(0xffffff0a) })
                                .text_color(if always_on_top { rgb(0x00d992) } else { rgb(0x8b949e) })
                                .hover(move |s| s.bg(if always_on_top { rgba(0x00d99240) } else { rgba(0xffffff15) }))
                                .on_click(move |_, _, cx| {
                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        let p = c.plugins.entry(plugin_id.to_string()).or_insert_with(|| widget_core::PluginConfig {
                                            x: 0.0, y: 0.0, width: 300.0, height: 300.0, scale: 1.0, phys_x: 0, phys_y: 0, phys_w: 0, phys_h: 0, always_on_top: false, mouse_passthrough: false, pinned_to_desktop: false, loaded: true, enabled: true
                                        });
                                        p.always_on_top = !always_on_top;
                                    });
                                    let hwnd = widget_core::get_plugin_hwnd(plugin_id);
                                    widget_core::set_window_always_on_top(hwnd, !always_on_top);
                                    widget_core::save_config_now(cx);
                                    cx.refresh_windows();
                                })
                                .child(div().text_sm().font_weight(FontWeight::MEDIUM).child("置顶"))
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("{}-ghost", plugin_id)))
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .p(px(6.0))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .bg(if mouse_passthrough { rgba(0x00d99230) } else { rgba(0xffffff0a) })
                                .text_color(if mouse_passthrough { rgb(0x00d992) } else { rgb(0x8b949e) })
                                .hover(move |s| s.bg(if mouse_passthrough { rgba(0x00d99240) } else { rgba(0xffffff15) }))
                                .on_click(move |_, _, cx| {
                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        let p = c.plugins.entry(plugin_id.to_string()).or_insert_with(|| widget_core::PluginConfig {
                                            x: 0.0, y: 0.0, width: 300.0, height: 300.0, scale: 1.0, phys_x: 0, phys_y: 0, phys_w: 0, phys_h: 0, always_on_top: false, mouse_passthrough: false, pinned_to_desktop: false, loaded: true, enabled: true
                                        });
                                        p.mouse_passthrough = !mouse_passthrough;
                                    });
                                    let hwnd = widget_core::get_plugin_hwnd(plugin_id);
                                    if hwnd != 0 {
                                        unsafe {
                                            use windows_sys::Win32::UI::WindowsAndMessaging::{
                                                GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_TRANSPARENT, WS_EX_LAYERED,
                                            };
                                            let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                                            SetWindowLongW(
                                                hwnd,
                                                GWL_EXSTYLE,
                                                if !mouse_passthrough {
                                                    style | WS_EX_TRANSPARENT as i32 | WS_EX_LAYERED as i32
                                                } else {
                                                    style & !(WS_EX_TRANSPARENT as i32 | WS_EX_LAYERED as i32)
                                                },
                                            );
                                        }
                                    }
                                    widget_core::save_config_now(cx);
                                    cx.refresh_windows();
                                })
                                .child(div().text_sm().font_weight(FontWeight::MEDIUM).child("穿透"))
                        )
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .children(has_settings.then(|| {
                            Button::new(("btn-settings", kind as usize), "")
                                .variant(ButtonVariant::Ghost)
                                .icon(gpui_component::IconName::Settings)
                                .on_click(move |_, _, cx| {
                                    let cb = cx.try_global::<widget_core::OpenPluginSettingsCallback>().cloned();
                                    if let Some(cb) = cb {
                                        cb.0(cx, plugin_id);
                                    }
                                })
                        }))
                        .child(
                            Button::new(("btn-load", kind as usize), load_label)
                                .variant(ButtonVariant::Outline)
                                .on_click(move |_, _, cx| {
                                    let next_loaded = !cx
                                        .try_global::<widget_core::UIState>()
                                        .is_none_or(|s| s.is_plugin_loaded(plugin_id));
                                    cx.update_global::<widget_core::UIState, _>(|s, _| {
                                        s.plugin_loaded.insert(plugin_id.to_string(), next_loaded);
                                        if !next_loaded {
                                            s.plugin_enabled.insert(plugin_id.to_string(), false);
                                        } else {
                                            s.plugin_enabled.insert(plugin_id.to_string(), true);
                                        }
                                    });

                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        let cfg = c.plugins.entry(plugin_id.to_string()).or_insert_with(|| widget_core::PluginConfig {
                                            x: 0.0,
                                            y: 0.0,
                                            width: 0.0,
                                            height: 0.0,
                                            scale: 1.0,
                                            phys_x: 0,
                                            phys_y: 0,
                                            phys_w: 0,
                                            phys_h: 0,
                                            always_on_top: false,
                                            mouse_passthrough: false,
                                            pinned_to_desktop: false,
                                            loaded: next_loaded,
                                            enabled: next_loaded,
                                        });
                                        cfg.loaded = next_loaded;
                                        if !next_loaded {
                                            cfg.enabled = false;
                                        } else {
                                            cfg.enabled = true;
                                        }
                                    });
                                    widget_core::save_config_now(cx);

                                    let plugin_id_string = plugin_id.to_string();
                                    if let Some(cb) = cx.try_global::<widget_core::TogglePluginCallback>().cloned() {
                                        cx.defer(move |cx| {
                                            (cb.0)(cx, &plugin_id_string, next_loaded);
                                        });
                                    }

                                    cx.refresh_windows();
                                }),
                        )
                        .child(
                            Button::new(("btn-enable", kind as usize), enable_label)
                                .variant(if is_enabled { ButtonVariant::Secondary } else { ButtonVariant::Default })
                                .on_click(move |_, _, cx| {
                                    let is_loaded = cx.try_global::<widget_core::UIState>().is_none_or(|s| s.is_plugin_loaded(plugin_id));
                                    if !is_loaded {
                                        return;
                                    }
                                    let next_enabled = !cx
                                        .try_global::<widget_core::UIState>()
                                        .is_none_or(|s| s.is_plugin_enabled(plugin_id));
                                    cx.update_global::<widget_core::UIState, _>(|s, _| {
                                        s.plugin_enabled.insert(plugin_id.to_string(), next_enabled);
                                    });

                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        if let Some(cfg) = c.plugins.get_mut(plugin_id) {
                                            cfg.enabled = next_enabled;
                                        }
                                    });
                                    widget_core::save_config_now(cx);

                                    let hwnd = widget_core::get_plugin_hwnd(plugin_id);
                                    if hwnd != 0 {
                                        unsafe {
                                            if next_enabled {
                                                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW);
                                            } else {
                                                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE);
                                            }
                                        }
                                    }
                                    cx.refresh_windows();
                                }),
                        ),
                )
        )
}
