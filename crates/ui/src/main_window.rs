use gpui::*;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::IsZoomed;

use crate::components::update_modal::render_update_modal;
use crate::pages::dashboard::render_dashboard_content;
use crate::pages::settings::{render_settings_page, SettingsTab};
use crate::pages::widgets::render_widgets_content;
use crate::sidebar::render_sidebar;
use crate::titlebar::render_titlebar;

use gpui_component::input::{InputEvent, InputState};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavPage {
    Dashboard,
    Widgets,
    Settings,
}

pub struct MainWindow {
    pub is_maximized: bool,
    pub nav_page: NavPage,
    pub settings_tab: SettingsTab,
    pub settings_search_input: Option<Entity<InputState>>,
    pub settings_collapsed_groups: [bool; 3],
    pub settings_anim_tokens: [u32; 3],
    pub widgets_filter: crate::pages::widgets::WidgetsFilter,
    pub widgets_anim_token: u32,
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
            settings_tab: SettingsTab::About,
            settings_search_input: None,
            settings_collapsed_groups: [false; 3],
            settings_anim_tokens: [0; 3],
            widgets_filter: crate::pages::widgets::WidgetsFilter::All,
            widgets_anim_token: 0,
        }
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.settings_search_input.is_none() {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("搜索设置..."));
            cx.subscribe(
                &input,
                |_this: &mut Self, _input: Entity<InputState>, event: &InputEvent, cx| {
                    if let InputEvent::Change = event {
                        cx.notify();
                    }
                },
            )
            .detach();
            self.settings_search_input = Some(input);
        }
        let is_visible = cx
            .try_global::<widget_core::UIState>()
            .is_none_or(|s| s.is_visible);
        let is_edit_mode = cx
            .try_global::<widget_core::UIState>()
            .is_some_and(|s| s.is_edit_mode);
        let nav_page = self.nav_page;
        let settings_tab = self.settings_tab;

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

        let mut root = div()
            .relative()
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
                    .overflow_hidden()
                    .min_h_0()
                    .child(render_sidebar(nav_page, cx))
                    .child(match nav_page {
                        NavPage::Dashboard => div()
                            .id("page-scroll")
                            .flex_1()
                            .h_full()
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .p(px(24.0))
                            .gap(px(20.0))
                            .children(render_dashboard_content(is_edit_mode, cx))
                            .into_any_element(),
                        NavPage::Widgets => div()
                            .id("page-scroll")
                            .flex_1()
                            .h_full()
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .p(px(24.0))
                            .gap(px(20.0))
                            .children(render_widgets_content(
                                self.widgets_filter,
                                self.widgets_anim_token,
                                cx,
                            ))
                            .into_any_element(),
                        NavPage::Settings => render_settings_page(
                            settings_tab,
                            &self.settings_search_input,
                            self.settings_collapsed_groups,
                            self.settings_anim_tokens,
                            cx,
                        )
                        .into_any_element(),
                    }),
            );

        // 顶层挂载更新提示弹窗
        if let Some(modal) = render_update_modal(cx) {
            root = root.child(modal);
        }

        root.into_any_element()
    }
}
