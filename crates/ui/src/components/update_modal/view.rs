use crate::components::button::{Button, ButtonVariant};
use crate::update::{
    apply_update_and_restart, download_update, MainWindowUpdateBridge, UpdateStatus,
};
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::IconName;

/// 独立更新弹窗主视图
#[derive(Default)]
pub struct UpdateModalView;

impl UpdateModalView {
    /// 创建独立更新弹窗视图实例
    pub fn new() -> Self {
        Self
    }
}

impl Render for UpdateModalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = cx
            .try_global::<MainWindowUpdateBridge>()
            .map(|b| b.status.clone())
            .unwrap_or(UpdateStatus::Idle);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x181818))
            .border_1()
            .border_color(rgb(0x3d3a39))
            .rounded(px(10.0))
            .shadow_lg()
            .overflow_hidden()
            .child(render_modal_titlebar(&status))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .p(px(20.0))
                    .gap(px(16.0))
                    .child(render_modal_content(&status)),
            )
    }
}

use super::titlebar::render_modal_titlebar;

/// 渲染弹窗内容区域
fn render_modal_content(status: &UpdateStatus) -> impl IntoElement {
    match status {
        UpdateStatus::Available {
            download_url,
            release_notes,
            is_installer,
            ..
        } => render_available_content(download_url, release_notes, *is_installer),
        UpdateStatus::Downloading(percent) => render_downloading_content(*percent),
        UpdateStatus::ReadyToRestart {
            new_exe_path,
            is_installer,
        } => render_ready_content(new_exe_path, *is_installer),
        UpdateStatus::Error(err) => render_error_content(err),
        _ => div()
            .flex()
            .items_center()
            .justify_center()
            .flex_1()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x8b949e))
                    .child("当前已是最新版本"),
            )
            .into_any_element(),
    }
}

fn render_available_content(url: &str, notes: &str, is_installer: bool) -> AnyElement {
    let download_url = url.to_string();
    let release_notes = notes.to_string();

    div()
        .flex()
        .flex_col()
        .flex_1()
        .justify_between()
        .gap(px(14.0))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
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
                        .flex_1()
                        .min_h(px(160.0))
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
                                .child(release_notes),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .justify_end()
                .items_center()
                .gap(px(12.0))
                .pt(px(4.0))
                .child(
                    Button::new("update-modal-later", "稍后提醒")
                        .variant(ButtonVariant::Ghost)
                        .on_click(|_, win, cx| {
                            win.remove_window();
                            cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
                                bridge.dismissed = true;
                                bridge.update_window = None;
                            });
                        }),
                )
                .child(
                    Button::new("update-modal-download", "立即更新")
                        .variant(ButtonVariant::Default)
                        .icon(IconName::ArrowDown)
                        .on_click(move |_, _, cx| {
                            download_update(download_url.clone(), is_installer, cx);
                        }),
                ),
        )
        .into_any_element()
}

fn render_downloading_content(percent: u8) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .justify_center()
        .gap(px(20.0))
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xf2f2f2))
                        .child("正在下载更新包，请稍候..."),
                )
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x00d992))
                        .child(format!("{}%", percent)),
                ),
        )
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
                        .w(relative(percent as f32 / 100.0))
                        .bg(rgb(0x00d992))
                        .rounded(px(4.0)),
                ),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .pt(px(12.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x8b949e))
                        .child("下载完成后可直接一键重启完成更新"),
                )
                .child(
                    Button::new("update-modal-hide", "后台下载")
                        .variant(ButtonVariant::Ghost)
                        .on_click(|_, win, cx| {
                            win.remove_window();
                            cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
                                bridge.dismissed = true;
                                bridge.update_window = None;
                            });
                        }),
                ),
        )
        .into_any_element()
}

fn render_ready_content(new_exe_path: &std::path::Path, is_installer: bool) -> AnyElement {
    let path_clone = new_exe_path.to_path_buf();
    div()
        .flex()
        .flex_col()
        .flex_1()
        .justify_between()
        .gap(px(16.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .pt(px(8.0))
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xf2f2f2))
                        .child("新版本已下载完毕"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x8b949e))
                        .line_height(relative(1.5))
                        .child("更新包已就绪。点击下方按钮将自动安装更新并重启 Widget RS。"),
                ),
        )
        .child(
            div()
                .flex()
                .justify_end()
                .items_center()
                .gap(px(12.0))
                .child(
                    Button::new("update-modal-install-later", "稍后重启")
                        .variant(ButtonVariant::Ghost)
                        .on_click(|_, win, cx| {
                            win.remove_window();
                            cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
                                bridge.dismissed = true;
                                bridge.update_window = None;
                            });
                        }),
                )
                .child(
                    Button::new("update-modal-install-now", "重启并更新")
                        .variant(ButtonVariant::Default)
                        .icon(IconName::ArrowRight)
                        .on_click(move |_, _, cx| {
                            apply_update_and_restart(&path_clone, is_installer, cx);
                        }),
                ),
        )
        .into_any_element()
}

fn render_error_content(err: &str) -> AnyElement {
    let err_text = err.to_string();
    div()
        .flex()
        .flex_col()
        .flex_1()
        .justify_between()
        .gap(px(16.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xf85149))
                        .child("检查或下载更新失败"),
                )
                .child(div().text_sm().text_color(rgb(0x8b949e)).child(err_text)),
        )
        .child(
            div().flex().justify_end().child(
                Button::new("update-modal-close-err", "关闭")
                    .variant(ButtonVariant::Ghost)
                    .on_click(|_, win, cx| {
                        win.remove_window();
                        cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
                            bridge.dismissed = true;
                            bridge.update_window = None;
                        });
                    }),
            ),
        )
        .into_any_element()
}
