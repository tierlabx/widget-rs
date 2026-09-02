use crate::components::button::{Button, ButtonVariant};
use crate::update::{
    apply_update_and_restart, dismiss_update_modal, download_update, MainWindowUpdateBridge,
    UpdateStatus,
};
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::IconName;

/// 渲染更新提示弹窗（当有可用更新或正在下载/安装时展示）
pub fn render_update_modal(cx: &mut App) -> Option<AnyElement> {
    let bridge = cx.try_global::<MainWindowUpdateBridge>()?;
    if bridge.dismissed {
        return None;
    }

    match &bridge.status {
        UpdateStatus::Available {
            version,
            download_url,
            release_notes,
            is_installer,
        } => {
            let version_str = version.clone();
            let url = download_url.clone();
            let notes = release_notes.clone();
            let installer_flag = *is_installer;

            Some(
                div()
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x00000088))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(480.0))
                            .max_h(px(520.0))
                            .bg(rgb(0x181818))
                            .border_1()
                            .border_color(rgb(0x3d3a39))
                            .rounded(px(12.0))
                            .shadow_lg()
                            .p(px(24.0))
                            .gap(px(16.0))
                            // 标题栏
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(div().text_color(rgb(0x00d992)).child(
                                                gpui_component::Icon::new(IconName::ArrowDown),
                                            ))
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0xf2f2f2))
                                                    .child(format!("发现新版本 v{}", version_str)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .id("close-update-modal-btn")
                                            .cursor_pointer()
                                            .p(px(4.0))
                                            .rounded(px(4.0))
                                            .text_color(rgb(0x8b949e))
                                            .hover(|s| {
                                                s.bg(rgba(0xffffff15)).text_color(rgb(0xf2f2f2))
                                            })
                                            .on_click(|_, _, cx| {
                                                dismiss_update_modal(cx);
                                            })
                                            .child(gpui_component::Icon::new(IconName::Close)),
                                    ),
                            )
                            // 更新内容
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0x8b949e))
                                            .child("更新内容"),
                                    )
                                    .child(
                                        div()
                                            .id("update-notes-scroll")
                                            .w_full()
                                            .max_h(px(240.0))
                                            .overflow_y_scrollbar()
                                            .p(px(12.0))
                                            .bg(rgb(0x101010))
                                            .border_1()
                                            .border_color(rgb(0x2d2a29))
                                            .rounded(px(8.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0xd1d5db))
                                                    .line_height(relative(1.5))
                                                    .child(notes),
                                            ),
                                    ),
                            )
                            // 底部操作区
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .items_center()
                                    .gap(px(12.0))
                                    .pt(px(8.0))
                                    .child(
                                        Button::new("update-modal-later", "稍后提醒")
                                            .variant(ButtonVariant::Ghost)
                                            .on_click(|_, _, cx| {
                                                dismiss_update_modal(cx);
                                            }),
                                    )
                                    .child(
                                        Button::new("update-modal-download", "立即更新")
                                            .variant(ButtonVariant::Default)
                                            .icon(IconName::ArrowDown)
                                            .on_click(move |_, _, cx| {
                                                download_update(url.clone(), installer_flag, cx);
                                            }),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
        UpdateStatus::Downloading(percent) => {
            let pct = *percent;
            Some(
                div()
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x00000088))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(420.0))
                            .bg(rgb(0x181818))
                            .border_1()
                            .border_color(rgb(0x3d3a39))
                            .rounded(px(12.0))
                            .shadow_lg()
                            .p(px(24.0))
                            .gap(px(16.0))
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xf2f2f2))
                                            .child("正在下载更新包..."),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x00d992))
                                            .child(format!("{}%", pct)),
                                    ),
                            )
                            // 进度条背景与滑块
                            .child(
                                div()
                                    .w_full()
                                    .h(px(8.0))
                                    .bg(rgb(0x2d2a29))
                                    .rounded(px(4.0))
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(relative(pct as f32 / 100.0))
                                            .bg(rgb(0x00d992))
                                            .rounded(px(4.0)),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x8b949e))
                                            .child("请稍候，下载完成后可直接重启完成更新"),
                                    )
                                    .child(
                                        Button::new("update-modal-hide", "后台下载")
                                            .variant(ButtonVariant::Ghost)
                                            .on_click(|_, _, cx| {
                                                dismiss_update_modal(cx);
                                            }),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
        UpdateStatus::ReadyToRestart {
            new_exe_path,
            is_installer,
        } => {
            let path_clone = new_exe_path.clone();
            let installer_flag = *is_installer;
            Some(
                div()
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x00000088))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(420.0))
                            .bg(rgb(0x181818))
                            .border_1()
                            .border_color(rgb(0x3d3a39))
                            .rounded(px(12.0))
                            .shadow_lg()
                            .p(px(24.0))
                            .gap(px(16.0))
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child("新版本准备就绪"),
                            )
                            .child(
                                div().text_sm().text_color(rgb(0x8b949e)).child(
                                    "新版本已准备完毕，点击下方按钮将自动安全更新并重启应用。",
                                ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .items_center()
                                    .gap(px(12.0))
                                    .pt(px(8.0))
                                    .child(
                                        Button::new("update-modal-install-later", "稍后重启")
                                            .variant(ButtonVariant::Ghost)
                                            .on_click(|_, _, cx| {
                                                dismiss_update_modal(cx);
                                            }),
                                    )
                                    .child(
                                        Button::new("update-modal-install-now", "重启并更新")
                                            .variant(ButtonVariant::Default)
                                            .icon(IconName::ArrowRight)
                                            .on_click(move |_, _, cx| {
                                                apply_update_and_restart(
                                                    &path_clone,
                                                    installer_flag,
                                                    cx,
                                                );
                                            }),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
        _ => None,
    }
}
