use gpui::prelude::FluentBuilder;
use gpui::*;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::IsZoomed;

use crate::pages::dashboard::render_dashboard_content;
use crate::pages::settings::render_settings_content;
use crate::pages::widgets::render_widgets_content;
use crate::sidebar::render_sidebar;
use crate::titlebar::render_titlebar;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavPage {
    Dashboard,
    Widgets,
    Settings,
}

pub struct MainWindow {
    pub is_maximized: bool,
    pub nav_page: NavPage,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            is_maximized: false,
            nav_page: NavPage::Dashboard,
        }
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_visible = cx
            .try_global::<widget_core::UIState>()
            .is_none_or(|s| s.is_visible);
        let is_edit_mode = cx
            .try_global::<widget_core::UIState>()
            .is_some_and(|s| s.is_edit_mode);
        let nav_page = self.nav_page;

        let is_maximized = if let Ok(h) = window.window_handle() {
            if let RawWindowHandle::Win32(h) = h.as_raw() {
                unsafe { IsZoomed(h.hwnd.get()) != 0 }
            } else {
                false
            }
        } else {
            false
        };

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
            .child(render_titlebar(window, cx, self.is_maximized))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .overflow_hidden() // Layout 统一处理滚动遮罩
                    .min_h_0() // 防止 flex 子项撑开父级
                    .child(render_sidebar(nav_page, cx))
                    .child(
                        // 统一的可滚动页面容器
                        div()
                            .id("page-scroll")
                            .flex_1()
                            .h_full()
                            .overflow_y_scroll() // Layout 层负责滚动
                            .flex()
                            .flex_col()
                            .p(px(24.0))
                            .gap(px(20.0))
                            .children(match nav_page {
                                NavPage::Dashboard => render_dashboard_content(is_edit_mode, cx),
                                NavPage::Widgets => render_widgets_content(cx),
                                NavPage::Settings => render_settings_content(cx),
                            }),
                    ),
            )
            .into_any_element()
    }
}
