use gpui::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ShowWindow, SW_HIDE, SW_RESTORE, IsZoomed, SetForegroundWindow
};

pub struct MainWindow {
    is_maximized: bool,
}

impl MainWindow {
    pub fn new() -> Self {
        Self { is_maximized: false }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_visible = cx.try_global::<widget_core::UIState>().map_or(true, |s| s.is_visible);
        let is_edit_mode = cx.try_global::<widget_core::UIState>().map_or(false, |s| s.is_edit_mode);

        let is_maximized = if let Ok(handle) = _window.window_handle() {
            if let RawWindowHandle::Win32(h) = handle.as_raw() {
                unsafe { IsZoomed(h.hwnd.get() as isize) != 0 }
            } else { false }
        } else { false };

        if self.is_maximized != is_maximized {
            self.is_maximized = is_maximized;
            cx.notify();
        }

        if !is_visible {
            return div().bg(rgba(0x00000000)).into_any_element();
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgba(0x050507f2))
            .border_1()
            .border_color(rgb(0x3d3a39))
            .child(self.render_titlebar(_window, cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .child(self.render_sidebar())
                    .child(self.render_main_content(is_edit_mode, cx))
            )
            .into_any_element()
    }
}

impl MainWindow {
    fn render_titlebar(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                        if let Ok(handle) = window.window_handle() {
                            if let RawWindowHandle::Win32(h) = handle.as_raw() {
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
                        div().flex().items_center().gap(px(10.0)).ml(px(16.0))
                        .child(div().text_color(rgb(0x00d992)).child(gpui_component::Icon::new(gpui_component::IconName::WindowMaximize)))
                        .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xf2f2f2)).child("Widget RS"))
                    )
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
                            .w(px(32.0)).h(px(32.0)).rounded(px(6.0))
                            .flex().justify_center().items_center()
                            .id("min-btn")
                            .hover(|s| s.bg(rgba(0xffffff1a)))
                            .on_click(|_, window, _| { window.minimize_window(); })
                            .child(div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(gpui_component::IconName::Minus)))
                    )
                    .child(
                        div()
                            .w(px(32.0)).h(px(32.0)).rounded(px(6.0))
                            .flex().justify_center().items_center()
                            .id("max-btn")
                            .hover(|s| s.bg(rgba(0xffffff1a)))
                            .on_click(move |_, window, cx| {
                                let mut hwnd_opt = None;
                                if let Ok(handle) = window.window_handle() {
                                    if let RawWindowHandle::Win32(h) = handle.as_raw() {
                                        unsafe {
                                            let hwnd = h.hwnd.get() as isize;
                                            hwnd_opt = Some(hwnd);
                                            if IsZoomed(hwnd) != 0 {
                                                windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SYSCOMMAND, windows_sys::Win32::UI::WindowsAndMessaging::SC_RESTORE as usize, 0);
                                            } else {
                                                windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SYSCOMMAND, windows_sys::Win32::UI::WindowsAndMessaging::SC_MAXIMIZE as usize, 0);
                                            }
                                        }
                                    }
                                }
                                let app_cx: &mut gpui::App = cx;
                                app_cx.spawn(async move |async_cx| {
                                    // 连续多次刷新以覆盖 Windows 的窗口动画时间 (大约 200-250ms)
                                    for _ in 0..6 {
                                        async_cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
                                        let _ = async_cx.update(|cx| {
                                            cx.refresh_windows();
                                        });
                                    }
                                    // 动画结束后，模拟一次系统级的重新聚焦，强制 GPUI 更新视图边界
                                    if let Some(hwnd) = hwnd_opt {
                                        unsafe {
                                            SetForegroundWindow(hwnd);
                                        }
                                    }
                                }).detach();
                            })
                            .child(
                                div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(if self.is_maximized { gpui_component::IconName::Minimize } else { gpui_component::IconName::Maximize }))
                            )
                    )
                    .child(
                        div()
                            .w(px(32.0)).h(px(32.0)).rounded(px(6.0))
                            .flex().justify_center().items_center()
                            .id("close-btn")
                            .hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                            .on_click(|_, window, cx| {
                                cx.update_global::<widget_core::UIState, _>(|state, _| {
                                    state.is_visible = false;
                                });
                                if let Ok(handle) = window.window_handle() {
                                    if let RawWindowHandle::Win32(h) = handle.as_raw() {
                                        unsafe { ShowWindow(h.hwnd.get() as isize, SW_HIDE); }
                                    }
                                }
                            })
                            .child(div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(gpui_component::IconName::Close)))
                    )
            )
    }

    fn render_sidebar(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(280.0))
            .h_full()
            .bg(rgb(0x101010))
            .border_r_1()
            .border_color(rgb(0x3d3a39))
            .child(
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .px(px(24.0))
                    .py(px(24.0))
                    .gap(px(16.0))
                    .child(
                        div()
                            .w(px(40.0)).h(px(40.0)).rounded(px(10.0))
                            .bg(rgb(0x050507)).border_1().border_color(rgba(0x00d99240))
                            .flex().justify_center().items_center()
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(rgb(0x00d992)).child("W"))
                    )
                    .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xf2f2f2)).child("Widget RS"))
            )
            .child(
                div()
                    .flex().flex_col().w_full().px(px(8.0)).gap(px(4.0))
                    .child(
                        div().flex().items_center().w_full().px(px(16.0)).py(px(10.0)).gap(px(16.0))
                        .rounded(px(8.0))
                        .bg(rgba(0x00d9921a))
                        .border_1().border_color(rgba(0x00d99220))
                        .child(div().text_color(rgb(0x00d992)).child(gpui_component::Icon::new(gpui_component::IconName::WindowMaximize)))
                        .child(div().text_sm().font_weight(FontWeight::MEDIUM).text_color(rgb(0xf2f2f2)).child("控制面板"))
                    )
                    .child(
                        div().flex().items_center().w_full().px(px(16.0)).py(px(10.0)).gap(px(16.0))
                        .rounded(px(8.0))
                        .child(div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(gpui_component::IconName::LayoutDashboard)))
                        .child(div().text_sm().font_weight(FontWeight::MEDIUM).text_color(rgb(0xb8b3b0)).child("小部件库"))
                    )
                    .child(
                        div().flex().items_center().w_full().px(px(16.0)).py(px(10.0)).gap(px(16.0))
                        .rounded(px(8.0))
                        .child(div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(gpui_component::IconName::Settings)))
                        .child(div().text_sm().font_weight(FontWeight::MEDIUM).text_color(rgb(0xb8b3b0)).child("设置"))
                    )
            )
            .child(div().flex_1())
            .child(
                div()
                    .flex().flex_col().w_full().p(px(16.0))
                    .child(
                        div().flex().items_center().w_full().p(px(12.0)).gap(px(8.0))
                        .bg(rgba(0x00d9920d)).border_1().border_color(rgba(0x00d99230)).rounded(px(8.0))
                        .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(rgb(0x00d992)))
                        .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(rgb(0x2fd6a1)).child("系统运行中"))
                    )
            )
    }

    fn render_main_content(&self, is_edit_mode: bool, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("main-scroll-container")
            .flex_1()
            .h_full()
            .overflow_y_scroll()
            .flex().flex_col()
            .p(px(24.0)).gap(px(16.0))
            .child(
                div()
                    .flex().justify_between().items_center().w_full()
                    .child(
                        div().flex().flex_col().gap(px(8.0))
                        .child(div().text_3xl().font_weight(FontWeight::BOLD).text_color(rgb(0xf2f2f2)).child("控制面板"))
                        .child(div().text_base().text_color(rgb(0xb8b3b0)).child("管理您的桌面小部件"))
                    )
                    .child(
                        div()
                            .flex().gap(px(16.0)).items_center()
                            .child(
                                div()
                                    .flex().items_center().justify_center().gap(px(8.0))
                                    .px(px(20.0)).py(px(12.0)).rounded(px(8.0))
                                    .bg(rgba(0x00d99218)).border_1().border_color(rgb(0x00d992))
                                    .id("edit-mode-btn")
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgba(0x00d99230)))
                                    .on_click(|_, _, cx| {
                                        cx.update_global::<widget_core::UIState, _>(|state, _| {
                                            state.is_edit_mode = !state.is_edit_mode;
                                        });
                                        cx.refresh_windows();
                                    })
                                    .child(div().text_base().text_color(rgb(0x2fd6a1)).child(if is_edit_mode { "✓" } else { "+" }))
                                    .child(
                                        div().text_base().font_weight(FontWeight::MEDIUM).text_color(rgb(0x2fd6a1))
                                        .child(if is_edit_mode { "完成排版" } else { "添加 / 排版" })
                                    )
                            )
                    )
            )
            .child(
                div()
                    .flex().w_full().gap(px(12.0))
                    .child(
                        div().flex_1().flex().items_center().gap(px(10.0)).px(px(16.0)).py(px(12.0))
                        .bg(rgba(0x00d9920d)).border_1().border_color(rgba(0x00d99225)).rounded(px(8.0))
                        .child(div().text_color(rgb(0x00d992)).child(gpui_component::Icon::new(gpui_component::IconName::Star)))
                        .child(
                            div().flex().flex_col().gap(px(2.0))
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(rgb(0xf2f2f2)).child("2"))
                            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(rgb(0x2fd6a1)).child("运行中"))
                        )
                    )
                    .child(
                        div().flex_1().flex().items_center().gap(px(10.0)).px(px(16.0)).py(px(12.0))
                        .bg(rgba(0xffffff06)).border_1().border_color(rgba(0x3d3a3960)).rounded(px(8.0))
                        .child(div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(gpui_component::IconName::CircleX)))
                        .child(
                            div().flex().flex_col().gap(px(2.0))
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(rgb(0xf2f2f2)).child("2"))
                            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(rgb(0x8b949e)).child("已停止"))
                        )
                    )
                    .child(
                        div().flex_1().flex().items_center().gap(px(10.0)).px(px(16.0)).py(px(12.0))
                        .bg(rgba(0xffffff06)).border_1().border_color(rgba(0x3d3a3960)).rounded(px(8.0))
                        .child(div().text_color(rgb(0xb8b3b0)).child(gpui_component::Icon::new(gpui_component::IconName::GalleryVerticalEnd)))
                        .child(
                            div().flex().flex_col().gap(px(2.0))
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(rgb(0xf2f2f2)).child("4"))
                            .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(rgb(0xb8b3b0)).child("小部件总数"))
                        )
                    )
            )
            .child(
                div()
                    .flex().flex_col().w_full().gap(px(16.0))
                    .child(
                        div().flex().justify_between().items_center().w_full()
                        .child(div().text_lg().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xf2f2f2)).child("已安装的小部件"))
                        .child(div().text_sm().text_color(rgb(0x8b949e)).child("4 个小部件"))
                    )
                    .child(
                        div().flex().flex_col().w_full().gap(px(16.0))
                        .child(
                            div().flex().w_full().gap(px(16.0))
                            .child(self.render_widget_card("便签", gpui_component::IconName::File, true, 0, cx))
                            .child(self.render_widget_card("待办事项", gpui_component::IconName::CircleCheck, true, 1, cx))
                        )
                        .child(
                            div().flex().w_full().gap(px(16.0))
                            .child(self.render_widget_card("时钟", gpui_component::IconName::LoaderCircle, false, 2, cx))
                            .child(self.render_widget_card("系统监控", gpui_component::IconName::SquareTerminal, false, 3, cx))
                        )
                    )
            )
    }

    fn render_widget_card(&self, title: &str, icon: gpui_component::IconName, is_running: bool, kind: u8, _cx: &mut Context<Self>) -> impl IntoElement {
        use crate::components::button::{Button, ButtonVariant};
        use crate::components::badge::{Badge, BadgeVariant};
        use crate::components::card::Card;

        let status_variant = if is_running { BadgeVariant::Default } else { BadgeVariant::Secondary };
        let status_text = if is_running { "运行中" } else { "已停止" };

        let action_btn1 = Button::new(("btn-settings", kind as usize), "设置")
            .variant(ButtonVariant::Ghost)
            .icon(gpui_component::IconName::Settings);

        let action_btn2 = if is_running {
            Button::new(("btn-toggle", kind as usize), "显示/隐藏")
                .variant(ButtonVariant::Secondary)
        } else {
            Button::new(("btn-start", kind as usize), "启动")
                .variant(ButtonVariant::Outline)
        };

        let preview_area = match kind {
            0 => {
                div()
                    .flex().flex_col().w_full().h_full().p(px(12.0)).rounded(px(6.0))
                    .bg(rgb(0xfef3c7)).border_1().border_color(rgba(0xf59e0b40))
                    .child(div().text_sm().text_color(rgb(0x78350f)).child("这是一个便签示例..."))
            },
            1 => {
                div()
                    .flex().flex_col().w_full().h_full().p(px(12.0)).gap(px(8.0)).rounded(px(6.0))
                    .bg(rgb(0x050507)).border_1().border_color(rgba(0x3d3a3940))
                    .child(
                        div().flex().items_center().w_full().p(px(10.0)).gap(px(10.0)).rounded(px(6.0))
                        .bg(rgba(0xffffff05)).border_1().border_color(rgba(0x3d3a3940))
                        .child(div().w(px(16.0)).h(px(16.0)).rounded_full().border_2().border_color(rgb(0x00d992)))
                        .child(div().text_sm().text_color(rgb(0xf2f2f2)).child("完成项目设计"))
                    )
                    .child(
                        div().flex().items_center().w_full().p(px(10.0)).gap(px(10.0)).rounded(px(6.0))
                        .bg(rgba(0xffffff05)).border_1().border_color(rgba(0x3d3a3940))
                        .child(div().w(px(16.0)).h(px(16.0)).rounded_full().bg(rgb(0x00d992)))
                        .child(div().text_sm().text_color(rgb(0x8b949e)).child("编写文档"))
                    )
            },
            2 => {
                div()
                    .flex().flex_col().justify_center().items_center().w_full().h_full().p(px(12.0)).rounded(px(6.0))
                    .bg(rgb(0x050507)).border_1().border_color(rgba(0x3d3a3940))
                    .child(div().text_3xl().font_weight(FontWeight::LIGHT).text_color(rgb(0x3d3a39)).child("--:--"))
                    .child(div().text_sm().text_color(rgb(0x8b949e)).child("2026 · 04 · 24"))
            },
            _ => {
                div()
                    .flex().flex_col().justify_center().items_center().w_full().h_full().p(px(12.0)).gap(px(8.0)).rounded(px(6.0))
                    .bg(rgb(0x050507)).border_1().border_color(rgba(0x3d3a3940))
                    .child(div().text_color(rgba(0x3d3a3960)).child(gpui_component::Icon::new(gpui_component::IconName::SquareTerminal).size(px(32.0))))
                    .child(div().text_sm().text_color(rgba(0x3d3a3980)).child("点击启动以显示系统信息"))
            }
        };

        Card::new()
            .fixed_height(px(210.0))
            .header(
                div()
                    .flex().justify_between().items_center().w_full()
                    .child(
                        div().flex().items_center().gap(px(8.0))
                        .child(
                            div().flex().justify_center().items_center().w(px(32.0)).h(px(32.0)).rounded(px(6.0))
                            .bg(rgba(0x00d9921a)).border_1().border_color(rgba(0x00d99240))
                            .child(div().text_color(if is_running { rgb(0x00d992) } else { rgb(0x8b949e) }).child(gpui_component::Icon::new(icon)))
                        )
                        .child(div().text_lg().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xf2f2f2)).child(title.to_string()))
                    )
                    .child(Badge::new(status_text).variant(status_variant).show_dot(true))
            )
            .content(preview_area)
            .footer(
                div().flex().w_full().justify_end().gap(px(8.0))
                .child(action_btn1)
                .child(action_btn2)
            )
    }
}
