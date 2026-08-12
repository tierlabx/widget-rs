use crate::components::toggle::toggle_switch;
use crate::layout::{page_header, section_title, settings_card, settings_row};
use crate::update::{check_for_update, download_update, MainWindowUpdateBridge, UpdateStatus};
use gpui::*;

pub fn render_settings_content(
    cx: &mut Context<crate::main_window::MainWindow>,
) -> Vec<gpui::AnyElement> {
    let auto_start = cx
        .try_global::<widget_core::AppConfig>()
        .is_some_and(|c| c.auto_start);

    let auto_check_update = cx
        .try_global::<widget_core::AppConfig>()
        .is_some_and(|c| c.auto_check_update);

    let update_status = cx
        .try_global::<MainWindowUpdateBridge>()
        .map(|bridge| &bridge.status)
        .unwrap_or(&UpdateStatus::Idle);

    vec![
        div()
            .flex()
            .justify_between()
            .items_center()
            .w_full()
            .child(page_header("设置", "配置小部件全局行为"))
            .into_any_element(),
        settings_card()
            .child(
                settings_row(false)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child("开机自启动"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x8b949e))
                                    .child("系统启动时自动运行应用"),
                            ),
                    )
                    .child(toggle_switch("auto-start", auto_start, move |val, cx| {
                        cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                            c.auto_start = val;
                        });
                        if let Ok(exe_path) = std::env::current_exe() {
                            if let Some(exe_str) = exe_path.to_str() {
                                let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                                if let Ok(run_key) = hkcu.open_subkey_with_flags(
                                    "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                                    winreg::enums::KEY_ALL_ACCESS,
                                ) {
                                    if val {
                                        let exe_path_quoted = format!("\"{}\"", exe_str);
                                        let _ = run_key.set_value("WidgetRS", &exe_path_quoted);
                                    } else {
                                        let _ = run_key.delete_value("WidgetRS");
                                    }
                                    let _ = run_key.delete_value("Widget RS");
                                }
                            }
                        }
                        widget_core::save_config_now(cx);
                    })),
            )
            .into_any_element(),
        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(section_title("更新"))
            .child(
                settings_card()
                    .child(
                        settings_row(true)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xf2f2f2))
                                            .child("自动检查更新"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x8b949e))
                                            .child("应用启动时自动检查新版本"),
                                    ),
                            )
                            .child(toggle_switch(
                                "auto-check-update",
                                auto_check_update,
                                move |val, cx| {
                                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                        c.auto_check_update = val;
                                    });
                                    widget_core::save_config_now(cx);
                                },
                            )),
                    )
                    .child(
                        settings_row(false)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xf2f2f2))
                                            .child("检查更新"),
                                    )
                                    .child(match update_status {
                                        UpdateStatus::Idle
                                        | UpdateStatus::Checking
                                        | UpdateStatus::Downloading(_) => div()
                                            .text_sm()
                                            .text_color(rgb(0x8b949e))
                                            .child("立即手动检查"),
                                        UpdateStatus::Available { version, .. } => div()
                                            .text_sm()
                                            .text_color(rgb(0x00d992))
                                            .child(format!("发现新版本: v{}", version)),
                                        UpdateStatus::ReadyToInstall(_) => div()
                                            .text_sm()
                                            .text_color(rgb(0x00d992))
                                            .child("下载完成，等待安装"),
                                        UpdateStatus::UpToDate => div()
                                            .text_sm()
                                            .text_color(rgb(0x8b949e))
                                            .child("已是最新版本"),
                                        UpdateStatus::Error(e) => div()
                                            .text_sm()
                                            .text_color(rgb(0xe81123))
                                            .child(format!("检查失败: {}", e)),
                                    }),
                            )
                            .child(match update_status {
                                UpdateStatus::Idle
                                | UpdateStatus::UpToDate
                                | UpdateStatus::Error(_) => div()
                                    .id("check-update-btn")
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .px(px(16.0))
                                    .py(px(8.0))
                                    .rounded(px(6.0))
                                    .bg(rgba(0xffffff0a))
                                    .border_1()
                                    .border_color(rgba(0xffffff1a))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgba(0xffffff15)))
                                    .on_click(|_, _, cx| {
                                        check_for_update(cx);
                                    })
                                    .child(div().text_color(rgb(0x8b949e)).child(
                                        gpui_component::Icon::new(
                                            gpui_component::IconName::LoaderCircle,
                                        ),
                                    ))
                                    .child(
                                        div().text_sm().text_color(rgb(0xf2f2f2)).child("检查更新"),
                                    )
                                    .into_any_element(),
                                UpdateStatus::Checking => div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .px(px(16.0))
                                    .py(px(8.0))
                                    .rounded(px(6.0))
                                    .bg(rgba(0xffffff05))
                                    .border_1()
                                    .border_color(rgba(0xffffff10))
                                    .child(div().text_color(rgb(0x00d992)).child(
                                        gpui_component::Icon::new(
                                            gpui_component::IconName::LoaderCircle,
                                        ),
                                    ))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x8b949e))
                                            .child("检查中..."),
                                    )
                                    .into_any_element(),
                                UpdateStatus::Available { download_url, .. } => {
                                    let url = download_url.clone();
                                    div()
                                        .id("download-update-btn")
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .px(px(16.0))
                                        .py(px(8.0))
                                        .rounded(px(6.0))
                                        .bg(rgba(0x00d9921a))
                                        .border_1()
                                        .border_color(rgba(0x00d99240))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgba(0x00d99230)))
                                        .on_click(move |_, _, cx| {
                                            download_update(url.clone(), cx);
                                        })
                                        .child(div().text_color(rgb(0x00d992)).child(
                                            gpui_component::Icon::new(
                                                gpui_component::IconName::ArrowDown,
                                            ),
                                        ))
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(0x00d992))
                                                .font_weight(FontWeight::BOLD)
                                                .child("立即更新"),
                                        )
                                        .into_any_element()
                                }
                                UpdateStatus::Downloading(percent) => div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .px(px(16.0))
                                    .py(px(8.0))
                                    .rounded(px(6.0))
                                    .bg(rgba(0x00d9921a))
                                    .border_1()
                                    .border_color(rgba(0x00d99240))
                                    .child(div().text_color(rgb(0x00d992)).child(
                                        gpui_component::Icon::new(
                                            gpui_component::IconName::ArrowDown,
                                        ),
                                    ))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x00d992))
                                            .font_weight(FontWeight::BOLD)
                                            .child(format!("下载中 {}%", percent)),
                                    )
                                    .into_any_element(),
                                UpdateStatus::ReadyToInstall(path) => {
                                    let path_clone = path.clone();
                                    div()
                                        .id("install-update-btn")
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .px(px(16.0))
                                        .py(px(8.0))
                                        .rounded(px(6.0))
                                        .bg(rgba(0x00d9921a))
                                        .border_1()
                                        .border_color(rgba(0x00d99240))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgba(0x00d99230)))
                                        .on_click(move |_, _, cx| {
                                            let _ = std::process::Command::new(&path_clone).spawn();
                                            cx.quit();
                                        })
                                        .child(div().text_color(rgb(0x00d992)).child(
                                            gpui_component::Icon::new(
                                                gpui_component::IconName::ArrowRight,
                                            ),
                                        ))
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(0x00d992))
                                                .font_weight(FontWeight::BOLD)
                                                .child("立即安装"),
                                        )
                                        .into_any_element()
                                }
                            }),
                    ),
            )
            .into_any_element(),
        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(section_title("关于"))
            .child(
                settings_card()
                    .child(
                        settings_row(true)
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child("Widget-RS"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x8b949e))
                                    .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                            ),
                    )
                    .child(
                        settings_row(true)
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child("开源地址"),
                            )
                            .child(
                                div()
                                    .id("github-link")
                                    .text_sm()
                                    .text_color(rgb(0x00d992))
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(rgb(0x2fd6a1)))
                                    .child("github.com/tierlabx/widget-rs")
                                    .on_click(|_, _, _| {
                                        let _ = open::that("https://github.com/tierlabx/widget-rs");
                                    }),
                            ),
                    )
                    .child(
                        settings_row(false)
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child("许可证"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x8b949e))
                                    .child("MIT License"),
                            ),
                    ),
            )
            .into_any_element(),
    ]
}
