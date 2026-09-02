use crate::components::toggle::toggle_switch;
use crate::layout::{section_title, settings_card, settings_row};
use crate::update::{
    apply_update_and_restart, check_for_update, download_update, MainWindowUpdateBridge,
    UpdateStatus,
};
use gpui::*;
use gpui_component::IconName;

pub fn render_update_section(cx: &mut Context<crate::main_window::MainWindow>) -> impl IntoElement {
    let auto_check_update = cx
        .try_global::<widget_core::AppConfig>()
        .is_some_and(|c| c.auto_check_update);

    let update_status = cx
        .try_global::<MainWindowUpdateBridge>()
        .map(|bridge| &bridge.status)
        .unwrap_or(&UpdateStatus::Idle);

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
                                        .child("启动时自动联网检查是否有新版本可用"),
                                ),
                        )
                        .child(toggle_switch(
                            "auto-check-update",
                            auto_check_update,
                            |val, cx| {
                                cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                    c.auto_check_update = val;
                                });
                                cx.refresh_windows();
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
                                .child(rarest_status_label(update_status)),
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
                                .child(
                                    div()
                                        .text_color(rgb(0x8b949e))
                                        .child(gpui_component::Icon::new(IconName::LoaderCircle)),
                                )
                                .child(mention_button_label("检查更新"))
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
                                .child(
                                    div()
                                        .text_color(rgb(0x00d992))
                                        .child(gpui_component::Icon::new(IconName::LoaderCircle)),
                                )
                                .child(div().text_sm().text_color(rgb(0x8b949e)).child("检查中..."))
                                .into_any_element(),
                            UpdateStatus::Available {
                                download_url,
                                is_installer,
                                ..
                            } => {
                                let url = download_url.clone();
                                let installer_flag = *is_installer;
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
                                        download_update(url.clone(), installer_flag, cx);
                                    })
                                    .child(
                                        div()
                                            .text_color(rgb(0x00d992))
                                            .child(gpui_component::Icon::new(IconName::ArrowDown)),
                                    )
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
                                .child(
                                    div()
                                        .text_color(rgb(0x00d992))
                                        .child(gpui_component::Icon::new(IconName::ArrowDown)),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0x00d992))
                                        .font_weight(FontWeight::BOLD)
                                        .child(format!("下载中 {}%", percent)),
                                )
                                .into_any_element(),
                            UpdateStatus::ReadyToRestart {
                                new_exe_path,
                                is_installer,
                            } => {
                                let path_clone = new_exe_path.clone();
                                let installer_flag = *is_installer;
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
                                        apply_update_and_restart(&path_clone, installer_flag, cx);
                                    })
                                    .child(
                                        div()
                                            .text_color(rgb(0x00d992))
                                            .child(gpui_component::Icon::new(IconName::ArrowRight)),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x00d992))
                                            .font_weight(FontWeight::BOLD)
                                            .child("重启并更新"),
                                    )
                                    .into_any_element()
                            }
                        }),
                ),
        )
}

fn mention_button_label(text: &'static str) -> impl IntoElement {
    div().text_sm().text_color(rgb(0xf2f2f2)).child(text)
}

fn rarest_status_label(status: &UpdateStatus) -> impl IntoElement {
    match status {
        UpdateStatus::Idle | UpdateStatus::Checking | UpdateStatus::Downloading(_) => div()
            .text_sm()
            .text_color(rgb(0x8b949e))
            .child("立即手动检查"),
        UpdateStatus::Available { version, .. } => div()
            .text_sm()
            .text_color(rgb(0x00d992))
            .child(format!("发现新版本: v{}", version)),
        UpdateStatus::ReadyToRestart { .. } => div()
            .text_sm()
            .text_color(rgb(0x00d992))
            .child("新版本已准备就绪"),
        UpdateStatus::UpToDate => div()
            .text_sm()
            .text_color(rgb(0x8b949e))
            .child("已是最新版本"),
        UpdateStatus::Error(e) => div()
            .text_sm()
            .text_color(rgb(0xe81123))
            .child(format!("检查失败: {}", e)),
    }
}
