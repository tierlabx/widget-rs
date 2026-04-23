use gpui::*;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};

pub struct MainWindow;

impl MainWindow {
    pub fn new() -> Self {
        Self
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_visible = cx.try_global::<widget_core::UIState>().map_or(true, |s| s.is_visible);
        let is_edit_mode = cx.try_global::<widget_core::UIState>().map_or(false, |s| s.is_edit_mode);
        
        if !is_visible {
            return div().bg(rgba(0x00000000)).into_any_element();
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgba(0x050507f2)) // Abyss Black transparent
            .border_1()
            .border_color(rgb(0x3d3a39)) // border around the whole window
            .child(
                // Custom Titlebar
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .w_full()
                    .h(px(32.0))
                    .bg(rgb(0x101010)) // Titlebar background
                    .id("titlebar")
                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                        if let Ok(handle) = window.window_handle() {
                            if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
                                unsafe {
                                    windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                                    windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW(
                                        h.hwnd.get() as isize, 
                                        windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN, 
                                        windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize, 
                                        0
                                    );
                                }
                            }
                        }
                    })
                    .child(
                        // Title
                        div().text_sm().text_color(rgb(0xb8b3b0)).ml(px(16.0)).child("Widget RS 控制中心")
                    )
                    .child(
                        // Window Controls
                        div()
                            .flex()
                            .items_center()
                            .h_full()
                            .child(
                                div()
                                    .w(px(46.0))
                                    .h_full()
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .id("min-btn")
                                    .hover(|s| s.bg(rgba(0xffffff1a)))
                                    .on_click(|_, window, _| { window.minimize_window(); })
                                    .child(div().w(px(10.0)).h(px(1.0)).bg(rgb(0xb8b3b0)))
                            )
                            .child(
                                div()
                                    .w(px(46.0))
                                    .h_full()
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .id("max-btn")
                                    .hover(|s| s.bg(rgba(0xffffff1a)))
                                    .on_click(|_, window, _| { window.zoom_window(); })
                                    .child(div().w(px(10.0)).h(px(10.0)).border_1().border_color(rgb(0xb8b3b0)))
                            )
                            .child(
                                div()
                                    .w(px(46.0))
                                    .h_full()
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .id("close-btn")
                                    .hover(|s| s.bg(rgb(0xe81123)))
                                    .on_click(|_, window, cx| { 
                                        cx.update_global::<widget_core::UIState, _>(|state, _| {
                                            state.is_visible = false;
                                        });
                                        if let Ok(handle) = window.window_handle() {
                                            if let RawWindowHandle::Win32(h) = handle.as_raw() {
                                                unsafe {
                                                    ShowWindow(h.hwnd.get() as isize, SW_HIDE);
                                                }
                                            }
                                        }
                                    })
                                    .child(div().text_sm().text_color(rgb(0xb8b3b0)).child("✕"))
                            )
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .child(
                        // Sidebar
                        div()
                            .flex()
                            .flex_col()
                            .w(px(280.0))
                            .h_full()
                            .bg(rgb(0x101010)) // Carbon Surface
                            .border_r_1()
                            .border_color(rgb(0x3d3a39)) // Warm Charcoal
                            .child(
                                // sidebarHeader
                                div()
                                    .flex()
                                    .items_center()
                                    .w_full()
                                    .px(px(16.0))
                                    .py(px(24.0))
                                    .gap(px(16.0))
                                    .child(
                                        // logoIcon
                                        div()
                                            .w(px(40.0))
                                            .h(px(40.0))
                                            .rounded(px(8.0))
                                            .bg(rgb(0x050507))
                                            .border_1()
                                            .border_color(rgb(0x3d3a39))
                                            .flex()
                                            .justify_center()
                                            .items_center()
                                            .child(div().text_lg().text_color(rgb(0x00d992)).child("W"))
                                    )
                                    .child(
                                        // appTitle
                                        div()
                                            .text_xl()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xf2f2f2))
                                            .child("Widget RS")
                                    )
                            )
                            .child(
                                // sidebarNav
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .p(px(8.0))
                                    .gap(px(4.0))
                                    .child(
                                        // navItem1 (Active)
                                        div()
                                            .flex()
                                            .items_center()
                                            .w_full()
                                            .px(px(16.0))
                                            .py(px(12.0))
                                            .gap(px(16.0))
                                            .rounded(px(6.0))
                                            .bg(rgba(0x00d9921a)) // Emerald Signal transparent
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x00d992))
                                                    .child("控制台核心 (Main Console)")
                                            )
                                    )
                                    .child(
                                        // navItem2
                                        div()
                                            .flex()
                                            .items_center()
                                            .w_full()
                                            .px(px(16.0))
                                            .py(px(12.0))
                                            .gap(px(16.0))
                                            .rounded(px(6.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0xb8b3b0))
                                                    .child("组件插件库")
                                            )
                                    )
                            )
                            .child(div().flex_1()) // sidebarSpacer
                            .child(
                                // sidebarFooter
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .p(px(16.0))
                                    .child(
                                        // statusFrame
                                        div()
                                            .flex()
                                            .items_center()
                                            .w_full()
                                            .p(px(12.0))
                                            .gap(px(8.0))
                                            .bg(rgb(0x050507))
                                            .rounded(px(6.0))
                                            .child(
                                                div()
                                                    .w(px(8.0))
                                                    .h(px(8.0))
                                                    .rounded_full()
                                                    .bg(rgb(0x00d992))
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0xb8b3b0))
                                                    .child("系统运行正常")
                                            )
                                    )
                            )
                    )
                    .child(
                        // Main Content
                        div()
                            .flex_1()
                            .h_full()
                            .flex()
                            .flex_col()
                            .p(px(32.0))
                            .gap(px(32.0))
                            .child(
                                // headerSection
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .w_full()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .text_3xl()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0xf2f2f2))
                                                    .child("欢迎回到 Widget RS")
                                            )
                                    )
                            )
                            .child(
                                // widgetsSection
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .gap(px(16.0))
                                    .child(
                                        div()
                                            .flex()
                                            .justify_between()
                                            .items_center()
                                            .w_full()
                                            .child(
                                                div()
                                                    .text_xl()
                                                    .text_color(rgb(0xf2f2f2))
                                                    .child("已加载的独立窗口部件")
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .px(px(16.0))
                                                    .py(px(8.0))
                                                    .rounded(px(6.0))
                                                    .bg(if is_edit_mode { rgb(0x00d992) } else { rgb(0x3d3a39) })
                                                    .id("edit-mode-btn")
                                                    .cursor_pointer()
                                                    .on_click(|_, _window, cx| {
                                                        cx.update_global::<widget_core::UIState, _>(|state, _| {
                                                            state.is_edit_mode = !state.is_edit_mode;
                                                        });
                                                        cx.refresh_windows();
                                                    })
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(if is_edit_mode { rgb(0x050507) } else { rgb(0xf2f2f2) })
                                                            .font_weight(FontWeight::SEMIBOLD)
                                                            .child(if is_edit_mode { "退出编辑模式" } else { "开启编辑模式" })
                                                    )
                                            )
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .gap(px(16.0))
                                            .child(
                                                div()
                                                    .w(px(200.0))
                                                    .h(px(120.0))
                                                    .bg(rgb(0x101010))
                                                    .border_1()
                                                    .border_color(rgb(0x3d3a39))
                                                    .rounded(px(8.0))
                                                    .p(px(16.0))
                                                    .child(div().text_base().text_color(rgb(0xf2f2f2)).child("便签小部件"))
                                            )
                                            .child(
                                                div()
                                                    .w(px(200.0))
                                                    .h(px(120.0))
                                                    .bg(rgb(0x101010))
                                                    .border_1()
                                                    .border_color(rgb(0x3d3a39))
                                                    .rounded(px(8.0))
                                                    .p(px(16.0))
                                                    .child(div().text_base().text_color(rgb(0xf2f2f2)).child("待办事项小部件"))
                                            )
                                    )
                            )
                    )
            )
            .into_any_element()
    }
}

