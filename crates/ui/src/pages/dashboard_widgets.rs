use gpui::*;

use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::Card;
use crate::pages::dashboard_preview::render_preview;

/// 渲染控制面板已安装的小部件卡片（包含微缩预览、置顶、穿透、设置、加载、启用按钮）
#[allow(clippy::too_many_arguments)]
pub fn render_widget_card(
    title: &str,
    plugin_id: &str,
    icon: gpui_component::IconName,
    is_loaded: bool,
    is_enabled: bool,
    always_on_top: bool,
    mouse_passthrough: bool,
    has_settings: bool,
    _kind: u8,
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

    let preview = render_preview(plugin_id);
    let id_string = plugin_id.to_string();
    let title_string = title.to_string();

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
                                .child(title_string),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x8b949e))
                                .child(format!(
                                    "~{:.1} MB",
                                    estimated_memory as f64 / 1024.0 / 1024.0
                                )),
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
                        .child({
                            let pid = id_string.clone();
                            div()
                                .id(SharedString::from(format!("{}-pin", pid)))
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .p(px(6.0))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .bg(if always_on_top {
                                    rgba(0x00d99230)
                                } else {
                                    rgba(0xffffff0a)
                                })
                                .text_color(if always_on_top {
                                    rgb(0x00d992)
                                } else {
                                    rgb(0x8b949e)
                                })
                                .hover(move |s| {
                                    s.bg(if always_on_top {
                                        rgba(0x00d99240)
                                    } else {
                                        rgba(0xffffff15)
                                    })
                                })
                                .on_click(move |_, _, cx| {
                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        let p = c.plugins.entry(pid.clone()).or_insert_with(|| {
                                            widget_core::PluginConfig {
                                                x: 0.0,
                                                y: 0.0,
                                                width: 300.0,
                                                height: 300.0,
                                                scale: 1.0,
                                                phys_x: 0,
                                                phys_y: 0,
                                                phys_w: 0,
                                                phys_h: 0,
                                                always_on_top: false,
                                                mouse_passthrough: false,
                                                pinned_to_desktop: false,
                                                loaded: true,
                                                enabled: true,
                                            }
                                        });
                                        p.always_on_top = !always_on_top;
                                    });
                                    let hwnd = widget_core::get_plugin_hwnd(&pid);
                                    widget_core::set_window_always_on_top(hwnd, !always_on_top);
                                    widget_core::save_config_now(cx);
                                    cx.refresh_windows();
                                })
                                .child(div().text_xs().child("置顶"))
                        })
                        .child({
                            let pid = id_string.clone();
                            div()
                                .id(SharedString::from(format!("{}-pass", pid)))
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .p(px(6.0))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .bg(if mouse_passthrough {
                                    rgba(0x00d99230)
                                } else {
                                    rgba(0xffffff0a)
                                })
                                .text_color(if mouse_passthrough {
                                    rgb(0x00d992)
                                } else {
                                    rgb(0x8b949e)
                                })
                                .hover(move |s| {
                                    s.bg(if mouse_passthrough {
                                        rgba(0x00d99240)
                                    } else {
                                        rgba(0xffffff15)
                                    })
                                })
                                .on_click(move |_, _, cx| {
                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        let p = c.plugins.entry(pid.clone()).or_insert_with(|| {
                                            widget_core::PluginConfig {
                                                x: 0.0,
                                                y: 0.0,
                                                width: 300.0,
                                                height: 300.0,
                                                scale: 1.0,
                                                phys_x: 0,
                                                phys_y: 0,
                                                phys_w: 0,
                                                phys_h: 0,
                                                always_on_top: false,
                                                mouse_passthrough: false,
                                                pinned_to_desktop: false,
                                                loaded: true,
                                                enabled: true,
                                            }
                                        });
                                        p.mouse_passthrough = !mouse_passthrough;
                                    });
                                    let hwnd = widget_core::get_plugin_hwnd(&pid);
                                    widget_core::set_window_mouse_passthrough(
                                        hwnd,
                                        !mouse_passthrough,
                                    );
                                    widget_core::save_config_now(cx);
                                    cx.refresh_windows();
                                })
                                .child(div().text_xs().child("穿透"))
                        })
                        .child(if has_settings {
                            let pid = id_string.clone();
                            div()
                                .id(SharedString::from(format!("{}-settings", pid)))
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .p(px(6.0))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .bg(rgba(0xffffff0a))
                                .text_color(rgb(0x8b949e))
                                .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0xf2f2f2)))
                                .on_click(move |_, _, cx| {
                                    let cb = cx
                                        .try_global::<widget_core::OpenPluginSettingsCallback>()
                                        .map(|c| c.0.clone());
                                    if let Some(cb) = cb {
                                        cb(cx, &pid);
                                    }
                                })
                                .child(div().text_xs().child("设置"))
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        }),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child({
                            let pid = id_string.clone();
                            Button::new(format!("{}-load", pid), load_label)
                                .variant(if is_loaded {
                                    ButtonVariant::Outline
                                } else {
                                    ButtonVariant::Default
                                })
                                .on_click(move |_, _, cx| {
                                    let new_loaded = !is_loaded;
                                    cx.update_global::<widget_core::UIState, _>(|s, _| {
                                        s.plugin_loaded.insert(pid.clone(), new_loaded);
                                    });
                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        let p = c.plugins.entry(pid.clone()).or_insert_with(|| {
                                            widget_core::PluginConfig {
                                                x: 0.0,
                                                y: 0.0,
                                                width: 300.0,
                                                height: 300.0,
                                                scale: 1.0,
                                                phys_x: 0,
                                                phys_y: 0,
                                                phys_w: 0,
                                                phys_h: 0,
                                                always_on_top: false,
                                                mouse_passthrough: false,
                                                pinned_to_desktop: false,
                                                loaded: true,
                                                enabled: true,
                                            }
                                        });
                                        p.loaded = new_loaded;
                                    });
                                    let toggle_cb = cx
                                        .try_global::<widget_core::TogglePluginCallback>()
                                        .map(|c| c.0.clone());
                                    if let Some(cb) = toggle_cb {
                                        cb(cx, &pid, new_loaded);
                                    }
                                    widget_core::save_config_now(cx);
                                    cx.refresh_windows();
                                })
                        })
                        .child({
                            let pid = id_string.clone();
                            Button::new(format!("{}-enable", pid), enable_label)
                                .variant(ButtonVariant::Ghost)
                                .on_click(move |_, _, cx| {
                                    let new_enabled = !is_enabled;
                                    cx.update_global::<widget_core::UIState, _>(|s, _| {
                                        s.plugin_enabled.insert(pid.clone(), new_enabled);
                                    });
                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        let p = c.plugins.entry(pid.clone()).or_insert_with(|| {
                                            widget_core::PluginConfig {
                                                x: 0.0,
                                                y: 0.0,
                                                width: 300.0,
                                                height: 300.0,
                                                scale: 1.0,
                                                phys_x: 0,
                                                phys_y: 0,
                                                phys_w: 0,
                                                phys_h: 0,
                                                always_on_top: false,
                                                mouse_passthrough: false,
                                                pinned_to_desktop: false,
                                                loaded: true,
                                                enabled: true,
                                            }
                                        });
                                        p.enabled = new_enabled;
                                    });
                                    widget_core::save_config_now(cx);
                                    cx.refresh_windows();
                                })
                        }),
                ),
        )
}
