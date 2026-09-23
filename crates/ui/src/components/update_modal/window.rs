use super::view::UpdateModalView;
use crate::update::MainWindowUpdateBridge;
use gpui::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// 弹出独立更新弹窗窗口
pub fn open_update_window(cx: &mut App) {
    if let Some(bridge) = cx.try_global::<MainWindowUpdateBridge>() {
        if let Some(existing_window) = bridge.update_window {
            let still_open = existing_window
                .update(cx, |_, win, _| {
                    if let Ok(wh) = win.window_handle() {
                        if let RawWindowHandle::Win32(h) = wh.as_raw() {
                            unsafe {
                                windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(
                                    h.hwnd.get(),
                                );
                                windows_sys::Win32::UI::WindowsAndMessaging::BringWindowToTop(
                                    h.hwnd.get(),
                                );
                            }
                        }
                    }
                })
                .is_ok();

            if still_open {
                return;
            }
        }
    }

    let options = WindowOptions {
        titlebar: None,
        window_background: WindowBackgroundAppearance::Opaque,
        kind: WindowKind::PopUp,
        is_resizable: false,
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(480.0), px(400.0)),
            cx,
        ))),
        ..Default::default()
    };

    let Ok(window_handle) = cx.open_window(options, |window, cx| {
        let view = cx.new(|_| UpdateModalView::new());
        cx.new(|cx| gpui_component::Root::new(view, window, cx))
    }) else {
        return;
    };

    let any_handle: AnyWindowHandle = window_handle.into();
    let _ = window_handle.update(cx, |_, win, _| {
        if let Ok(wh) = win.window_handle() {
            if let RawWindowHandle::Win32(h) = wh.as_raw() {
                unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(h.hwnd.get());
                    windows_sys::Win32::UI::WindowsAndMessaging::BringWindowToTop(h.hwnd.get());
                }
            }
        }
    });

    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
        bridge.update_window = Some(any_handle);
        bridge.dismissed = false;
    });
}

/// 关闭独立更新弹窗窗口
pub fn close_update_window(cx: &mut App) {
    let window_handle = cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
        bridge.dismissed = true;
        bridge.update_window.take()
    });

    if let Some(handle) = window_handle {
        let _ = handle.update(cx, |_, window, _| {
            window.remove_window();
        });
    }

    cx.refresh_windows();
}
