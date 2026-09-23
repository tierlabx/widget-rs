use crate::update::{MainWindowUpdateBridge, UpdateStatus};
use gpui::*;
use gpui_component::IconName;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// 渲染弹窗标题栏（含拖拽移动区域与关闭按钮）
pub(super) fn render_modal_titlebar(status: &UpdateStatus) -> impl IntoElement {
    let title_text = match status {
        UpdateStatus::Available { version, .. } => format!("发现新版本 v{}", version),
        UpdateStatus::Downloading(_) => "正在下载更新包...".to_string(),
        UpdateStatus::ReadyToRestart { .. } => "新版本准备就绪".to_string(),
        UpdateStatus::Error(_) => "更新出现错误".to_string(),
        _ => "软件更新".to_string(),
    };

    div()
        .flex()
        .justify_between()
        .items_center()
        .w_full()
        .h(px(44.0))
        .flex_shrink_0()
        .bg(rgb(0x141414))
        .border_b_1()
        .border_color(rgb(0x2d2a29))
        .child(
            div()
                .flex()
                .flex_1()
                .items_center()
                .h_full()
                .pl(px(16.0))
                .id("update-modal-titlebar-drag")
                .on_mouse_down(MouseButton::Left, |_, win, _| {
                    if let Ok(h) = win.window_handle() {
                        if let RawWindowHandle::Win32(h) = h.as_raw() {
                            unsafe {
                                windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                                windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                    h.hwnd.get(),
                                    windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN,
                                    windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize,
                                    0,
                                );
                            }
                        }
                    }
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_color(rgb(0x00d992))
                                .child(gpui_component::Icon::new(IconName::ArrowDown)),
                        )
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xf2f2f2))
                                .child(title_text),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(44.0))
                .h_full()
                .hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                .text_color(rgb(0x8b949e))
                .cursor_pointer()
                .id("close-update-window-btn")
                .on_click(|_, win, cx| {
                    win.remove_window();
                    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
                        bridge.dismissed = true;
                        bridge.update_window = None;
                    });
                    cx.defer(|_| {
                        widget_core::trim_process_memory();
                    });
                })
                .child(gpui_component::Icon::new(IconName::Close)),
        )
}
