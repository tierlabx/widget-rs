use gpui::prelude::FluentBuilder;
use gpui::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsZoomed, SetForegroundWindow, ShowWindow, SW_HIDE,
};

pub fn render_titlebar(
    _window: &mut Window,
    _cx: &mut Context<crate::main_window::MainWindow>,
    is_maximized: bool,
) -> impl IntoElement {
    let is_max = is_maximized;
    div()
        .flex()
        .justify_between()
        .items_center()
        .w_full()
        .h(px(48.0))
        .bg(rgb(0x101010))
        .border_b_1()
        .border_color(rgb(0x3d3a39))
        .child(
            div()
                .flex()
                .flex_1()
                .items_center()
                .h_full()
                .id("titlebar-drag")
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
                        .gap(px(10.0))
                        .ml(px(16.0))
                        .child(img("logos/icon.png").w(px(24.0)).h(px(24.0))),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .h_full()
                .gap(px(2.0))
                .pr(px(16.0))
                .child(
                    div()
                        .w(px(32.0))
                        .h(px(32.0))
                        .rounded(px(6.0))
                        .flex()
                        .justify_center()
                        .items_center()
                        .id("min-btn")
                        .hover(|s| s.bg(rgba(0xffffff1a)))
                        .on_click(|_, win, _| {
                            win.minimize_window();
                        })
                        .child(
                            div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(
                                gpui_component::IconName::Minus,
                            )),
                        ),
                )
                .child(
                    div()
                        .w(px(32.0))
                        .h(px(32.0))
                        .rounded(px(6.0))
                        .flex()
                        .justify_center()
                        .items_center()
                        .id("max-btn")
                        .hover(|s| s.bg(rgba(0xffffff1a)))
                        .on_click(move |_, win, cx| {
                            let mut hwnd_opt = None;
                            if let Ok(h) = win.window_handle() {
                                if let RawWindowHandle::Win32(h) = h.as_raw() {
                                    unsafe {
                                        let hwnd = h.hwnd.get();
                                        hwnd_opt = Some(hwnd);
                                        if IsZoomed(hwnd) != 0 {
                                            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                                hwnd,
                                                windows_sys::Win32::UI::WindowsAndMessaging::WM_SYSCOMMAND,
                                                windows_sys::Win32::UI::WindowsAndMessaging::SC_RESTORE
                                                    as usize,
                                                0,
                                            );
                                        } else {
                                            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                                hwnd,
                                                windows_sys::Win32::UI::WindowsAndMessaging::WM_SYSCOMMAND,
                                                windows_sys::Win32::UI::WindowsAndMessaging::SC_MAXIMIZE
                                                    as usize,
                                                0,
                                            );
                                        }
                                    }
                                }
                            }
                            let app_cx: &mut gpui::App = cx;
                            app_cx
                                .spawn(async move |async_cx| {
                                    for _ in 0..6 {
                                        async_cx
                                            .background_executor()
                                            .timer(std::time::Duration::from_millis(50))
                                            .await;
                                        let _ = async_cx.update(|cx| {
                                            cx.refresh_windows();
                                        });
                                    }
                                    if let Some(hwnd) = hwnd_opt {
                                        unsafe {
                                            SetForegroundWindow(hwnd);
                                        }
                                    }
                                })
                                .detach();
                        })
                        .child(
                            div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(
                                if is_max {
                                    gpui_component::IconName::Minimize
                                } else {
                                    gpui_component::IconName::Maximize
                                },
                            )),
                        ),
                )
                .child(
                    div()
                        .w(px(32.0))
                        .h(px(32.0))
                        .rounded(px(6.0))
                        .flex()
                        .justify_center()
                        .items_center()
                        .id("close-btn")
                        .hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                        .on_click(|_, win, cx| {
                            cx.update_global::<widget_core::UIState, _>(|s, _| {
                                s.is_visible = false;
                            });
                            if let Ok(h) = win.window_handle() {
                                if let RawWindowHandle::Win32(h) = h.as_raw() {
                                    unsafe {
                                        ShowWindow(h.hwnd.get(), SW_HIDE);
                                    }
                                }
                            }
                        })
                        .child(
                            div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(
                                gpui_component::IconName::Close,
                            )),
                        ),
                ),
        )
}
