use gpui::prelude::FluentBuilder;
use gpui::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsZoomed, SetForegroundWindow, ShowWindow, SW_HIDE, SW_SHOW,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavPage {
    Dashboard,
    Widgets,
    Settings,
}

pub struct MainWindow {
    is_maximized: bool,
    nav_page: NavPage,
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
            .map_or(true, |s| s.is_visible);
        let is_edit_mode = cx
            .try_global::<widget_core::UIState>()
            .map_or(false, |s| s.is_edit_mode);
        let nav_page = self.nav_page;

        let is_maximized = if let Ok(h) = window.window_handle() {
            if let RawWindowHandle::Win32(h) = h.as_raw() {
                unsafe { IsZoomed(h.hwnd.get() as isize) != 0 }
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
            .child(self.render_titlebar(window, cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .child(self.render_sidebar(nav_page, cx))
                    .child(match nav_page {
                        NavPage::Dashboard => {
                            self.render_dashboard(is_edit_mode, cx).into_any_element()
                        }
                        NavPage::Widgets => self.render_widgets_page().into_any_element(),
                        NavPage::Settings => self.render_settings_page(cx).into_any_element(),
                    }),
            )
            .into_any_element()
    }
}

impl MainWindow {
    fn render_titlebar(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let is_max = self.is_maximized;
        div().flex().justify_between().items_center().w_full().h(px(48.0))
            .bg(rgb(0x101010)).border_b_1().border_color(rgb(0x3d3a39))
            .child(
                div().flex().flex_1().items_center().h_full().id("titlebar-drag")
                    .on_mouse_down(MouseButton::Left, |_, win, _| {
                        if let Ok(h) = win.window_handle() {
                            if let RawWindowHandle::Win32(h) = h.as_raw() { unsafe {
                                windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                                windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                    h.hwnd.get() as isize,
                                    windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN,
                                    windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize, 0);
                            }}
                        }
                    })
                      .child(div().flex().items_center().gap(px(10.0)).ml(px(16.0))
                        .child(gpui_component::Icon::empty().path("logos/icon.svg").size(px(24.0)).text_color(rgb(0x00d992)))
                        )
            )
            .child(
                div().flex().items_center().h_full().gap(px(2.0)).pr(px(16.0))
                    .child(div().w(px(32.0)).h(px(32.0)).rounded(px(6.0)).flex().justify_center().items_center()
                        .id("min-btn").hover(|s| s.bg(rgba(0xffffff1a)))
                        .on_click(|_, win, _| { win.minimize_window(); })
                        .child(div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(gpui_component::IconName::Minus))))
                    .child(div().w(px(32.0)).h(px(32.0)).rounded(px(6.0)).flex().justify_center().items_center()
                        .id("max-btn").hover(|s| s.bg(rgba(0xffffff1a)))
                        .on_click(move |_, win, cx| {
                            let mut hwnd_opt = None;
                            if let Ok(h) = win.window_handle() {
                                if let RawWindowHandle::Win32(h) = h.as_raw() { unsafe {
                                    let hwnd = h.hwnd.get() as isize; hwnd_opt = Some(hwnd);
                                    if IsZoomed(hwnd) != 0 {
                                        windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SYSCOMMAND, windows_sys::Win32::UI::WindowsAndMessaging::SC_RESTORE as usize, 0);
                                    } else {
                                        windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SYSCOMMAND, windows_sys::Win32::UI::WindowsAndMessaging::SC_MAXIMIZE as usize, 0);
                                    }
                                }}
                            }
                            let app_cx: &mut gpui::App = cx;
                            app_cx.spawn(async move |async_cx| {
                                for _ in 0..6 {
                                    async_cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
                                    let _ = async_cx.update(|cx| { cx.refresh_windows(); });
                                }
                                if let Some(hwnd) = hwnd_opt { unsafe { SetForegroundWindow(hwnd); } }
                            }).detach();
                        })
                        .child(div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(
                            if is_max { gpui_component::IconName::Minimize } else { gpui_component::IconName::Maximize }))))
                    .child(div().w(px(32.0)).h(px(32.0)).rounded(px(6.0)).flex().justify_center().items_center()
                        .id("close-btn").hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                        .on_click(|_, win, cx| {
                            cx.update_global::<widget_core::UIState, _>(|s, _| { s.is_visible = false; });
                            if let Ok(h) = win.window_handle() {
                                if let RawWindowHandle::Win32(h) = h.as_raw() {
                                    unsafe { ShowWindow(h.hwnd.get() as isize, SW_HIDE); }
                                }
                            }
                        })
                        .child(div().text_color(rgb(0x8b949e)).child(gpui_component::Icon::new(gpui_component::IconName::Close))))
            )
    }

    fn render_sidebar(&self, nav_page: NavPage, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(280.0))
            .h_full()
            .bg(rgb(0x101010))
            .border_r_1()
            .border_color(rgb(0x3d3a39))
            .pt(px(24.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .px(px(8.0))
                    .gap(px(4.0))
                    .child(self.nav_item(
                        "控制面板",
                        gpui_component::IconName::WindowMaximize,
                        NavPage::Dashboard,
                        nav_page,
                        cx,
                    ))
                    .child(self.nav_item(
                        "小部件库",
                        gpui_component::IconName::LayoutDashboard,
                        NavPage::Widgets,
                        nav_page,
                        cx,
                    ))
                    .child(self.nav_item(
                        "设置",
                        gpui_component::IconName::Settings,
                        NavPage::Settings,
                        nav_page,
                        cx,
                    )),
            )
            .child(div().flex_1())
            .child(
                div().flex().flex_col().w_full().p(px(16.0)).child(
                    div()
                        .flex()
                        .items_center()
                        .w_full()
                        .p(px(12.0))
                        .gap(px(8.0))
                        .bg(rgba(0x00d9920d))
                        .border_1()
                        .border_color(rgba(0x00d99230))
                        .rounded(px(8.0))
                        .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(rgb(0x00d992)))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x2fd6a1))
                                .child("系统运行中"),
                        ),
                ),
            )
    }

    fn nav_item(
        &self,
        label: &'static str,
        icon: gpui_component::IconName,
        page: NavPage,
        current: NavPage,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = page == current;
        let handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
            this.nav_page = page;
            cx.notify();
        });
        div()
            .flex()
            .items_center()
            .w_full()
            .px(px(16.0))
            .py(px(10.0))
            .gap(px(16.0))
            .rounded(px(8.0))
            .when(active, |d: gpui::Div| {
                d.bg(rgba(0x00d9921a))
                    .border_1()
                    .border_color(rgba(0x00d99220))
            })
            .id(ElementId::Name(label.into()))
            .cursor_pointer()
            .hover(move |s: gpui::StyleRefinement| if !active { s.bg(rgba(0xffffff08)) } else { s })
            .on_click(handler)
            .child(
                div()
                    .text_color(if active { rgb(0x00d992) } else { rgb(0x8b949e) })
                    .child(gpui_component::Icon::new(icon)),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if active { rgb(0xf2f2f2) } else { rgb(0xb8b3b0) })
                    .child(label),
            )
    }

    fn render_dashboard(&self, is_edit_mode: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let sticky_visible = cx
            .try_global::<widget_core::UIState>()
            .map_or(true, |s| s.is_plugin_visible("sticky_widget"));
        let todo_visible = cx
            .try_global::<widget_core::UIState>()
            .map_or(true, |s| s.is_plugin_visible("todo_widget"));

        div()
            .id("main-scroll")
            .flex_1()
            .h_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .p(px(24.0))
            .gap(px(16.0))
            .child(
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
                                    .child("控制面板"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(rgb(0xb8b3b0))
                                    .child("管理您的桌面小部件"),
                            ),
                    )
                    .child(
                        div().flex().gap(px(16.0)).items_center().child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap(px(8.0))
                                .px(px(20.0))
                                .py(px(12.0))
                                .rounded(px(8.0))
                                .bg(rgba(0x00d99218))
                                .border_1()
                                .border_color(rgb(0x00d992))
                                .id("edit-mode-btn")
                                .cursor_pointer()
                                .hover(|s| s.bg(rgba(0x00d99230)))
                                .on_click(|_, _, cx| {
                                    cx.update_global::<widget_core::UIState, _>(|s, _| {
                                        s.is_edit_mode = !s.is_edit_mode;
                                    });
                                    cx.refresh_windows();
                                })
                                .child(
                                    div()
                                        .text_base()
                                        .text_color(rgb(0x2fd6a1))
                                        .child(if is_edit_mode { "✓" } else { "+" }),
                                )
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0x2fd6a1))
                                        .child(if is_edit_mode {
                                            "完成排版"
                                        } else {
                                            "添加 / 排版"
                                        }),
                                ),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .w_full()
                    .gap(px(12.0))
                    .child(self.stat_card(
                        gpui_component::IconName::Star,
                        "2",
                        "运行中",
                        rgb(0x00d992),
                        rgba(0x00d9920d),
                        rgba(0x00d99225),
                    ))
                    .child(self.stat_card(
                        gpui_component::IconName::CircleX,
                        "0",
                        "已停止",
                        rgb(0x8b949e),
                        rgba(0xffffff06),
                        rgba(0x3d3a3960),
                    ))
                    .child(self.stat_card(
                        gpui_component::IconName::GalleryVerticalEnd,
                        "2",
                        "小部件总数",
                        rgb(0xb8b3b0),
                        rgba(0xffffff06),
                        rgba(0x3d3a3960),
                    )),
            )
            .child(
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
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child("已安装的小部件"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x8b949e))
                                    .child("2 个小部件"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .gap(px(16.0))
                            .child(self.widget_card(
                                "便签",
                                "sticky_widget",
                                gpui_component::IconName::File,
                                sticky_visible,
                                0,
                            ))
                            .child(self.widget_card(
                                "待办事项",
                                "todo_widget",
                                gpui_component::IconName::CircleCheck,
                                todo_visible,
                                1,
                            )),
                    ),
            )
    }

    fn stat_card(
        &self,
        icon: gpui_component::IconName,
        num: &'static str,
        label: &'static str,
        ic: Rgba,
        bg: Rgba,
        border: Rgba,
    ) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .gap(px(10.0))
            .px(px(16.0))
            .py(px(12.0))
            .bg(bg)
            .border_1()
            .border_color(border)
            .rounded(px(8.0))
            .child(div().text_color(ic).child(gpui_component::Icon::new(icon)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xf2f2f2))
                            .child(num),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(ic)
                            .child(label),
                    ),
            )
    }

    fn widget_card(
        &self,
        title: &'static str,
        plugin_id: &'static str,
        icon: gpui_component::IconName,
        is_visible: bool,
        kind: u8,
    ) -> impl IntoElement {
        use crate::components::badge::{Badge, BadgeVariant};
        use crate::components::button::{Button, ButtonVariant};
        use crate::components::card::Card;

        let toggle_label: &'static str = if is_visible { "隐藏" } else { "显示" };

        let preview = if kind == 0 {
            div()
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .p(px(12.0))
                .rounded(px(6.0))
                .bg(rgb(0xfef3c7))
                .border_1()
                .border_color(rgba(0xf59e0b40))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x78350f))
                        .child("这是一个便签示例..."),
                )
        } else {
            div()
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .p(px(12.0))
                .gap(px(6.0))
                .rounded(px(6.0))
                .bg(rgb(0x050507))
                .border_1()
                .border_color(rgba(0x3d3a3940))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .p(px(8.0))
                        .rounded(px(4.0))
                        .bg(rgba(0xffffff05))
                        .child(
                            div()
                                .w(px(12.0))
                                .h(px(12.0))
                                .rounded_full()
                                .bg(rgb(0x00d992)),
                        )
                        .child(div().text_sm().text_color(rgb(0x8b949e)).child("编写文档")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .p(px(8.0))
                        .rounded(px(4.0))
                        .bg(rgba(0xffffff05))
                        .child(
                            div()
                                .w(px(12.0))
                                .h(px(12.0))
                                .rounded_full()
                                .border_2()
                                .border_color(rgb(0x00d992)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xf2f2f2))
                                .child("完成项目设计"),
                        ),
                )
        };

        Card::new()
            .fixed_height(px(210.0))
            .header(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .w(px(32.0))
                                    .h(px(32.0))
                                    .rounded(px(6.0))
                                    .bg(rgba(0x00d9921a))
                                    .border_1()
                                    .border_color(rgba(0x00d99240))
                                    .child(
                                        div()
                                            .text_color(rgb(0x00d992))
                                            .child(gpui_component::Icon::new(icon)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child(title),
                            ),
                    )
                    .child(
                        Badge::new("运行中")
                            .variant(BadgeVariant::Default)
                            .show_dot(true),
                    ),
            )
            .content(preview)
            .footer(
                div()
                    .flex()
                    .w_full()
                    .justify_end()
                    // 直接用 Win32 ShowWindow，不经过 GPUI 全局，彻底避免 RefCell 冲突
                    .child(
                        Button::new(("btn-toggle", kind as usize), toggle_label)
                            .variant(ButtonVariant::Secondary)
                            .on_click(move |_, _, cx| {
                                // 1. 更新 UIState 可见状态（&mut App，无 RefCell）
                                let next_visible = !cx
                                    .try_global::<widget_core::UIState>()
                                    .map_or(true, |s| s.is_plugin_visible(plugin_id));
                                cx.update_global::<widget_core::UIState, _>(|s, _| {
                                    s.plugin_visibility
                                        .insert(plugin_id.to_string(), next_visible);
                                });
                                // 2. 读 HWND（thread_local，无 RefCell 冲突）
                                let hwnd = widget_core::get_plugin_hwnd(plugin_id);
                                // 3. 直接调 Win32 API
                                if hwnd != 0 {
                                    unsafe {
                                        if next_visible {
                                            ShowWindow(hwnd, SW_SHOW);
                                        } else {
                                            ShowWindow(hwnd, SW_HIDE);
                                        }
                                    }
                                }
                                cx.refresh_windows();
                            }),
                    ),
            )
    }

    fn render_widgets_page(&self) -> impl IntoElement {
        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .p(px(24.0))
            .gap(px(16.0))
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
                            .child("小部件库"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0xb8b3b0))
                            .child("浏览和安装新的小部件"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .p(px(24.0))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgb(0x3d3a39))
                    .bg(rgba(0xffffff04))
                    .child(
                        div()
                            .flex()
                            .justify_center()
                            .items_center()
                            .h(px(80.0))
                            .child(
                                div().text_color(rgba(0x3d3a3980)).child(
                                    gpui_component::Icon::new(
                                        gpui_component::IconName::LayoutDashboard,
                                    )
                                    .size(px(48.0)),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .text_center()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0xf2f2f2))
                            .child("即将推出更多小部件"),
                    )
                    .child(
                        div()
                            .w_full()
                            .text_center()
                            .text_sm()
                            .text_color(rgb(0x8b949e))
                            .child("更多实用小部件正在开发中，敬请期待"),
                    ),
            )
    }

    fn render_settings_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let always_on_top = cx
            .try_global::<widget_core::AppConfig>()
            .map_or(false, |c| c.always_on_top);
        let mouse_passthrough = cx
            .try_global::<widget_core::AppConfig>()
            .map_or(false, |c| c.mouse_passthrough);

        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .p(px(24.0))
            .gap(px(20.0))
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
                            .child("设置"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0xb8b3b0))
                            .child("配置小部件全局行为"),
                    ),
            )
            .child(self.setting_toggle(
                "始终置顶",
                "所有小部件窗口始终显示在其他窗口之上",
                "always-on-top",
                always_on_top,
                move |val, cx| {
                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                        c.always_on_top = val;
                    });
                    // 直接用 thread_local HWND 批量置顶（无 RefCell 冲突）
                    for hwnd in widget_core::get_all_plugin_hwnds() {
                        if hwnd == 0 {
                            continue;
                        }
                        unsafe {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{
                                SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
                            };
                            let insert_after = if val { HWND_TOPMOST } else { HWND_NOTOPMOST };
                            SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                        }
                    }
                    cx.refresh_windows();
                },
            ))
            .child(self.setting_toggle(
                "鼠标穿透",
                "鼠标点击将穿透小部件，可点击到桌面和其他窗口",
                "mouse-passthrough",
                mouse_passthrough,
                move |val, cx| {
                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                        c.mouse_passthrough = val;
                    });
                    for hwnd in widget_core::get_all_plugin_hwnds() {
                        if hwnd == 0 {
                            continue;
                        }
                        unsafe {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{
                                GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_TRANSPARENT,
                            };
                            let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                            SetWindowLongW(
                                hwnd,
                                GWL_EXSTYLE,
                                if val {
                                    style | WS_EX_TRANSPARENT as i32
                                } else {
                                    style & !(WS_EX_TRANSPARENT as i32)
                                },
                            );
                        }
                    }
                    cx.refresh_windows();
                },
            ))
    }

    fn setting_toggle(
        &self,
        title: &'static str,
        desc: &'static str,
        id: &'static str,
        enabled: bool,
        on_toggle: impl Fn(bool, &mut App) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .justify_between()
            .items_center()
            .w_full()
            .p(px(20.0))
            .rounded(px(10.0))
            .bg(rgb(0x101010))
            .border_1()
            .border_color(rgb(0x3d3a39))
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
                            .child(title),
                    )
                    .child(div().text_sm().text_color(rgb(0x8b949e)).child(desc)),
            )
            .child(
                div()
                    .id(ElementId::Name(id.into()))
                    .cursor_pointer()
                    .w(px(48.0))
                    .h(px(26.0))
                    .rounded_full()
                    .bg(if enabled {
                        rgb(0x00d992)
                    } else {
                        rgb(0x3d3a39)
                    })
                    .flex()
                    .items_center()
                    .px(px(3.0))
                    .on_click(move |_, _, cx| {
                        on_toggle(!enabled, cx);
                    })
                    .child(
                        div()
                            .w(px(20.0))
                            .h(px(20.0))
                            .rounded_full()
                            .bg(rgb(0xffffff))
                            .when(enabled, |d: gpui::Div| d.ml(px(22.0))),
                    ),
            )
    }
}
