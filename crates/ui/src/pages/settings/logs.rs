use crate::layout::{page_header, settings_card};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::IconName;
use std::fs;
use std::path::Path;

pub fn render_logs_settings(
    cx: &mut Context<crate::main_window::MainWindow>,
) -> Vec<gpui::AnyElement> {
    let log_dir = widget_core::get_log_dir();
    let data_dir = widget_core::get_data_dir();
    let log_dir_str = log_dir.to_string_lossy().to_string();
    let data_dir_str = data_dir.to_string_lossy().to_string();

    let crash_log_path = log_dir.join("crash.log");
    let crash_log_exists = crash_log_path.exists();
    let crash_log_size_desc = if crash_log_exists {
        if let Ok(metadata) = fs::metadata(&crash_log_path) {
            let len = metadata.len();
            if len < 1024 {
                format!("{} B", len)
            } else if len < 1024 * 1024 {
                format!("{:.1} KB", len as f64 / 1024.0)
            } else {
                format!("{:.2} MB", len as f64 / (1024.0 * 1024.0))
            }
        } else {
            "未知大小".to_string()
        }
    } else {
        "暂无日志".to_string()
    };

    // 统计日志文件数量
    let log_files_count = if let Ok(entries) = fs::read_dir(&log_dir) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("log"))
            })
            .count()
    } else {
        0
    };

    let log_dir_for_open = log_dir.clone();
    let log_dir_for_copy = log_dir_str.clone();

    let data_dir_for_open = data_dir.clone();
    let data_dir_for_copy = data_dir_str.clone();

    let crash_log_for_open = crash_log_path.clone();
    let log_dir_for_clear = log_dir.clone();

    let clear_logs_handler = cx.listener(move |_this, _: &ClickEvent, _window, cx| {
        clear_all_logs(&log_dir_for_clear);
        cx.notify();
    });

    vec![
        div()
            .flex()
            .justify_between()
            .items_center()
            .w_full()
            .child(page_header(
                "运行日志",
                "查看本地运行与崩溃日志存储路径，支持快速打开与日志清理",
            ))
            .into_any_element(),
        // 日志目录卡片
        render_path_card(
            "日志存储目录",
            &format!("共包含 {} 个日志文件", log_files_count),
            &log_dir_str,
            "open-log-dir-btn",
            "copy-log-dir-btn",
            move || {
                let _ = open::that(&log_dir_for_open);
            },
            log_dir_for_copy,
        ),
        // 最新崩溃日志卡片
        settings_card()
            .p(px(20.0))
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child("异常与崩溃日志 (crash.log)"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if crash_log_exists {
                                        rgb(0xf2a154)
                                    } else {
                                        rgb(0x8b949e)
                                    })
                                    .child(if crash_log_exists {
                                        format!(
                                            "检测到崩溃记录文件 (文件大小: {})",
                                            crash_log_size_desc
                                        )
                                    } else {
                                        "当前未产生崩溃或异常错误日志，运行状态良好".to_string()
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .when(crash_log_exists, |d| {
                                d.child(action_btn(
                                    "open-crash-log-btn",
                                    "打开日志",
                                    IconName::ExternalLink,
                                    true,
                                    move |_, _, _| {
                                        let _ = open::that(&crash_log_for_open);
                                    },
                                ))
                                .child(
                                    div()
                                        .id("clear-logs-btn")
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .px(px(12.0))
                                        .py(px(6.0))
                                        .rounded(px(6.0))
                                        .bg(rgba(0xff555518))
                                        .border_1()
                                        .border_color(rgba(0xff555538))
                                        .text_color(rgb(0xff7777))
                                        .hover(|s| s.bg(rgba(0xff555530)))
                                        .on_click(clear_logs_handler)
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .child("清空日志"),
                                        ),
                                )
                            }),
                    ),
            )
            .into_any_element(),
        // 应用数据存储目录卡片
        render_path_card(
            "应用数据与数据库存储",
            "存放应用本地 SQLite 数据库、小部件配置与状态缓存",
            &data_dir_str,
            "open-data-dir-btn",
            "copy-data-dir-btn",
            move || {
                let _ = open::that(&data_dir_for_open);
            },
            data_dir_for_copy,
        ),
    ]
}

/// 路径类展示与操作通用卡片
fn render_path_card(
    title: &'static str,
    subtitle: &str,
    path_display: &str,
    open_btn_id: &'static str,
    copy_btn_id: &'static str,
    open_action: impl Fn() + 'static,
    path_for_copy: String,
) -> gpui::AnyElement {
    settings_card()
        .p(px(20.0))
        .gap(px(16.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xf2f2f2))
                                .child(title),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x8b949e))
                                .child(subtitle.to_string()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(action_btn(
                            open_btn_id,
                            "打开目录",
                            IconName::Folder,
                            true,
                            move |_, _, _| open_action(),
                        ))
                        .child(action_btn(
                            copy_btn_id,
                            "复制路径",
                            IconName::Copy,
                            false,
                            move |_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(
                                    path_for_copy.clone(),
                                ));
                            },
                        )),
                ),
        )
        .child(
            div()
                .w_full()
                .px(px(12.0))
                .py(px(8.0))
                .bg(rgb(0x141414))
                .border_1()
                .border_color(rgb(0x262626))
                .rounded(px(6.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x8b949e))
                        .child(path_display.to_string()),
                ),
        )
        .into_any_element()
}

/// 操作按钮封装
fn action_btn(
    id: &'static str,
    label: &'static str,
    icon: IconName,
    is_primary: bool,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let (bg, border, text_col, hover_bg) = if is_primary {
        (
            rgba(0x00d99218),
            rgba(0x00d99238),
            rgb(0x00d992),
            rgba(0x00d99230),
        )
    } else {
        (
            rgba(0x212121ff),
            rgba(0x353535ff),
            rgb(0xcccccc),
            rgba(0x2a2a2aff),
        )
    };

    div()
        .id(ElementId::Name(id.into()))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(12.0))
        .py(px(6.0))
        .rounded(px(6.0))
        .bg(bg)
        .border_1()
        .border_color(border)
        .text_color(text_col)
        .hover(move |s| s.bg(hover_bg))
        .on_click(handler)
        .child(gpui_component::Icon::new(icon))
        .child(div().text_xs().font_weight(FontWeight::MEDIUM).child(label))
}

/// 清除日志目录下所有 .log 文件
fn clear_all_logs(log_dir: &Path) {
    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("log"))
            {
                let _ = fs::remove_file(path);
            }
        }
    }
}
