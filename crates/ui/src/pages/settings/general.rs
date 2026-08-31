use crate::components::toggle::toggle_switch;
use crate::layout::{page_header, settings_card, settings_row};
use gpui::*;

pub fn render_general_settings(
    cx: &mut Context<crate::main_window::MainWindow>,
) -> Vec<gpui::AnyElement> {
    let auto_start = cx
        .try_global::<widget_core::AppConfig>()
        .is_some_and(|c| c.auto_start);

    vec![
        div()
            .flex()
            .justify_between()
            .items_center()
            .w_full()
            .child(page_header("通用设置", "配置开机自启动与桌面行为"))
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
                                    .child("系统启动时自动在后台运行 Widget-RS"),
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
    ]
}
