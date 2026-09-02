use gpui::*;
use std::time::Duration;

use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonVariant};

struct PluginTheme {
    icon_bg: Rgba,
    icon_border: Rgba,
    icon_color: Rgba,
    tags: &'static [&'static str],
}

fn get_plugin_theme(id: &str) -> PluginTheme {
    match id {
        "sticky_widget" => PluginTheme {
            icon_bg: rgba(0xf59e0b14),
            icon_border: rgba(0xf59e0b30),
            icon_color: rgb(0xfbbf24),
            tags: &["轻量便签", "随手速记", "置顶展示"],
        },
        "todo_widget" => PluginTheme {
            icon_bg: rgba(0x38bdf814),
            icon_border: rgba(0x38bdf830),
            icon_color: rgb(0x38bdf8),
            tags: &["极简清单", "目标聚焦", "高效待办"],
        },
        "stretchly_widget" => PluginTheme {
            icon_bg: rgba(0x00d99214),
            icon_border: rgba(0x00d99230),
            icon_color: rgb(0x00d992),
            tags: &["健康护眼", "定时休息", "防疲劳"],
        },
        "fences_widget" => PluginTheme {
            icon_bg: rgba(0xa855f714),
            icon_border: rgba(0xa855f730),
            icon_color: rgb(0xc084fc),
            tags: &["桌面收纳", "栅格分类", "毛玻璃"],
        },
        _ => PluginTheme {
            icon_bg: rgba(0x00d99214),
            icon_border: rgba(0x00d99230),
            icon_color: rgb(0x00d992),
            tags: &["桌面扩展", "即开即用", "极低占用"],
        },
    }
}

/// 渲染小部件市场卡片
#[allow(clippy::too_many_arguments)]
pub fn render_market_card(
    name: &'static str,
    id_str: &'static str,
    desc: &'static str,
    icon: gpui_component::IconName,
    version: &'static str,
    author: &'static str,
    estimated_memory: usize,
    is_loaded: bool,
    index: usize,
    anim_token: u32,
) -> impl IntoElement {
    let theme = get_plugin_theme(id_str);
    let mem_str = if estimated_memory > 0 {
        format!("~{:.1} MB", estimated_memory as f64 / 1024.0 / 1024.0)
    } else {
        "< 2.0 MB".to_string()
    };

    let card_content =
        div()
            .flex()
            .flex_col()
            .w(px(340.0))
            .h(px(240.0))
            .p(px(20.0))
            .justify_between()
            .bg(rgb(0x121215))
            .border_1()
            .border_color(rgb(0x27272a))
            .rounded(px(10.0))
            .hover(|s| s.border_color(rgba(0x00d99280)).bg(rgb(0x16171c)))
            // 头部：图标 + 标题 + 版本/作者
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(14.0))
                    .child(
                        div()
                            .w(px(46.0))
                            .h(px(46.0))
                            .rounded(px(10.0))
                            .bg(theme.icon_bg)
                            .border_1()
                            .border_color(theme.icon_border)
                            .flex()
                            .justify_center()
                            .items_center()
                            .child(
                                div()
                                    .text_color(theme.icon_color)
                                    .child(gpui_component::Icon::new(icon)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .w_full()
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xf4f4f5))
                                            .child(name),
                                    )
                                    .child(
                                        Badge::new(format!("v{}", version))
                                            .variant(BadgeVariant::Outline)
                                            .show_dot(false),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x71717a))
                                    .child(format!("by {}", author)),
                            ),
                    ),
            )
            // 中间：描述 + 特性标签芯片
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xa1a1aa))
                            .h(px(38.0))
                            .overflow_hidden()
                            .child(desc),
                    )
                    .child(div().flex().items_center().gap(px(6.0)).children(
                        theme.tags.iter().map(|tag| {
                            div()
                                .px(px(8.0))
                                .py(px(2.0))
                                .rounded(px(4.0))
                                .bg(rgba(0xffffff08))
                                .border_1()
                                .border_color(rgba(0xffffff0f))
                                .text_xs()
                                .text_color(rgb(0x8b949e))
                                .child(*tag)
                        }),
                    )),
            )
            // 底部：物理内存预估 + 获取/已安装按钮
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .w_full()
                    .pt(px(10.0))
                    .border_t_1()
                    .border_color(rgba(0xffffff0a))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(if is_loaded {
                                rgb(0x00d992)
                            } else {
                                rgb(0x71717a)
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x71717a))
                                    .child(format!("预估内存 {}", mem_str)),
                            ),
                    )
                    .child(
                        Button::new(id_str, if is_loaded { "已安装" } else { "获取" })
                            .variant(if is_loaded {
                                ButtonVariant::Secondary
                            } else {
                                ButtonVariant::Default
                            })
                            .icon(if is_loaded {
                                gpui_component::IconName::Check
                            } else {
                                gpui_component::IconName::Plus
                            })
                            .on_click(move |_, _, cx| {
                                if is_loaded {
                                    return;
                                }
                                cx.update_global::<widget_core::UIState, _>(|s, _| {
                                    s.plugin_loaded.insert(id_str.to_string(), true);
                                    s.plugin_enabled.insert(id_str.to_string(), true);
                                });

                                cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                    let cfg =
                                        c.plugins.entry(id_str.to_string()).or_insert_with(|| {
                                            widget_core::PluginConfig {
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
                                                loaded: true,
                                                enabled: true,
                                            }
                                        });
                                    cfg.loaded = true;
                                    cfg.enabled = true;
                                });
                                widget_core::save_config_now(cx);

                                let plugin_id_string = id_str.to_string();
                                if let Some(cb) = cx
                                    .try_global::<widget_core::TogglePluginCallback>()
                                    .cloned()
                                {
                                    cx.defer(move |cx| {
                                        (cb.0)(cx, &plugin_id_string, true);
                                    });
                                }

                                cx.refresh_windows();
                            }),
                    ),
            );

    // 阶梯式平滑入场动画（Staggered entrance animation）
    let anim_duration = 200 + (index as u64).min(8) * 35;
    card_content.with_animation(
        ElementId::Name(format!("market-anim-{}-{}-{}", id_str, index, anim_token).into()),
        Animation::new(Duration::from_millis(anim_duration)).with_easing(gpui::ease_in_out),
        move |el, delta| {
            let progress = delta.clamp(0.0, 1.0);
            let offset = (1.0 - progress) * 10.0;
            el.opacity(progress).mt(px(offset))
        },
    )
}
