use gpui::prelude::FluentBuilder;
use gpui::*;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsZoomed, SetForegroundWindow, ShowWindow, SW_HIDE,
};

fn get_private_memory_usage() -> usize {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::ProcessStatus::K32GetProcessMemoryInfo;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        #[allow(non_snake_case)]
        #[repr(C)]
        struct PROCESS_MEMORY_COUNTERS_EX2 {
            pub cb: u32,
            pub PageFaultCount: u32,
            pub PeakWorkingSetSize: usize,
            pub WorkingSetSize: usize,
            pub QuotaPeakPagedPoolUsage: usize,
            pub QuotaPagedPoolUsage: usize,
            pub QuotaPeakNonPagedPoolUsage: usize,
            pub QuotaNonPagedPoolUsage: usize,
            pub PagefileUsage: usize,
            pub PeakPagefileUsage: usize,
            pub PrivateUsage: usize,
            pub PrivateWorkingSetSize: usize,
            pub SharedCommitUsage: u64,
        }

        unsafe {
            let mut mem_counters: PROCESS_MEMORY_COUNTERS_EX2 = std::mem::zeroed();
            mem_counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX2>() as u32;
            if K32GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut mem_counters as *mut _ as *mut _,
                mem_counters.cb,
            ) != 0
            {
                return mem_counters.PrivateWorkingSetSize;
            }
        }
    }

    // Fallback to memory_stats for non-Windows
    memory_stats::memory_stats()
        .map(|s| s.physical_mem)
        .unwrap_or(0)
}

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
                        NavPage::Widgets => self.render_widgets_page(cx).into_any_element(),
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
                                    h.hwnd.get(),
                                    windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN,
                                    windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize, 0);
                            }}
                        }
                    })
                      .child(div().flex().items_center().gap(px(10.0)).ml(px(16.0))
                        .child(img("logos/icon.png").w(px(24.0)).h(px(24.0)))
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
                                    let hwnd = h.hwnd.get(); hwnd_opt = Some(hwnd);
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
                                    unsafe { ShowWindow(h.hwnd.get(), SW_HIDE); }
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
        let plugin_list = cx
            .try_global::<widget_core::PluginList>()
            .map(|list| list.0.clone())
            .unwrap_or_default();

        let plugins_info: Vec<_> = plugin_list
            .iter()
            .map(|meta| {
                let loaded = cx
                    .try_global::<widget_core::UIState>()
                    .is_none_or(|s| s.is_plugin_loaded(meta.id));
                let enabled = cx
                    .try_global::<widget_core::UIState>()
                    .is_none_or(|s| s.is_plugin_enabled(meta.id));
                let top = cx
                    .try_global::<widget_core::AppConfig>()
                    .and_then(|c| c.plugins.get(meta.id))
                    .is_some_and(|p| p.always_on_top);
                let pass = cx
                    .try_global::<widget_core::AppConfig>()
                    .and_then(|c| c.plugins.get(meta.id))
                    .is_some_and(|p| p.mouse_passthrough);

                let estimated_memory = meta.estimated_memory;

                (meta.clone(), loaded, enabled, top, pass, estimated_memory)
            })
            .collect();

        let total_widgets = plugins_info.len();
        let running_widgets = plugins_info
            .iter()
            .filter(|(_, l, e, _, _, _)| *l && *e)
            .count();
        let stopped_widgets = total_widgets - running_widgets;

        let total_mem = get_private_memory_usage();
        let mem_str = format!("{:.1} MB", total_mem as f64 / 1024.0 / 1024.0);

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
                                    let mut was_edit_mode = false;
                                    let mut is_edit = false;
                                    cx.update_global::<widget_core::UIState, _>(|s, _| {
                                        was_edit_mode = s.is_edit_mode;
                                        s.is_edit_mode = !s.is_edit_mode;
                                        is_edit = s.is_edit_mode;
                                    });
                                    widget_core::NATIVE_EDIT_MODE
                                        .store(is_edit, std::sync::atomic::Ordering::SeqCst);

                                    if was_edit_mode {
                                        if let Some(cb) =
                                            cx.try_global::<widget_core::SaveBoundsCallback>()
                                        {
                                            let cb = cb.0.clone();
                                            cb(cx);
                                        }
                                    }
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
                        running_widgets.to_string(),
                        "运行中",
                        rgb(0x00d992),
                        rgba(0x00d9920d),
                        rgba(0x00d99225),
                    ))
                    .child(self.stat_card(
                        gpui_component::IconName::CircleX,
                        stopped_widgets.to_string(),
                        "已停止",
                        rgb(0x8b949e),
                        rgba(0xffffff06),
                        rgba(0x3d3a3960),
                    ))
                    .child(self.stat_card(
                        gpui_component::IconName::GalleryVerticalEnd,
                        total_widgets.to_string(),
                        "小部件总数",
                        rgb(0xb8b3b0),
                        rgba(0xffffff06),
                        rgba(0x3d3a3960),
                    ))
                    .child(self.stat_card(
                        gpui_component::IconName::LayoutDashboard,
                        mem_str.clone(),
                        "主进程物理内存",
                        rgb(0x00d992), // matching green theme
                        rgba(0x00d9920d),
                        rgba(0x00d99225),
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
                                    .child(format!("{} 个小部件", total_widgets)),
                            ),
                    )
                    .child(div().flex().w_full().gap(px(16.0)).flex_wrap().children(
                        plugins_info.into_iter().enumerate().filter_map(
                            |(i, (meta, loaded, enabled, top, pass, mem))| {
                                loaded.then(|| {
                                    self.widget_card(
                                        meta.name, meta.id, meta.icon, loaded, enabled, top, pass,
                                        i as u8, mem,
                                    )
                                })
                            },
                        ),
                    )),
            )
    }

    fn stat_card(
        &self,
        icon: gpui_component::IconName,
        num: impl Into<SharedString>,
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
                            .child(num.into()),
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

    #[allow(clippy::too_many_arguments)]
    fn widget_card(
        &self,
        title: &'static str,
        plugin_id: &'static str,
        icon: gpui_component::IconName,
        is_loaded: bool,
        is_enabled: bool,
        always_on_top: bool,
        mouse_passthrough: bool,
        kind: u8,
        estimated_memory: usize,
    ) -> impl IntoElement {
        use crate::components::badge::{Badge, BadgeVariant};
        use crate::components::button::{Button, ButtonVariant};
        use crate::components::card::Card;

        let load_label: &'static str = if is_loaded { "卸载" } else { "加载" };
        let enable_label: &'static str = if is_enabled { "关闭" } else { "启用" };

        let status_badge = if !is_loaded {
            Badge::new("未加载")
                .variant(BadgeVariant::Outline)
                .show_dot(false)
        } else if is_enabled {
            Badge::new("运行中")
                .variant(BadgeVariant::Default)
                .show_dot(true)
        } else {
            Badge::new("已关闭")
                .variant(BadgeVariant::Secondary)
                .show_dot(true)
        };

        let preview = match plugin_id {
            "sticky_widget" => div()
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
                ),
            "todo_widget" => div()
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
                ),
            "stretchly_widget" => div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .w_full()
                .h_full()
                .p(px(12.0))
                .rounded(px(6.0))
                .bg(rgb(0x050507))
                .border_1()
                .border_color(rgba(0x3d3a3940))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w_full()
                        .h_full()
                        .bg(rgba(0x00d99210))
                        .rounded(px(4.0))
                        .border_1()
                        .border_color(rgba(0x00d99240))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x00d992))
                                .font_weight(FontWeight::BOLD)
                                .child("休息提醒 - 专注中..."),
                        ),
                ),
            _ => div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .w_full()
                .h_full()
                .p(px(12.0))
                .rounded(px(6.0))
                .bg(rgb(0x050507))
                .border_1()
                .border_color(rgba(0x3d3a3940))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x8b949e))
                        .child("桌面宠物 3D 渲染中..."),
                ),
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
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x8b949e))
                                    .child(format!("~{:.1} MB", estimated_memory as f64 / 1024.0 / 1024.0)),
                            ),
                    )
                    .child(status_badge),
            )
            .content(preview)
            .footer(
                div()
                    .flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .id(SharedString::from(format!("{}-pin", plugin_id)))
                                    .flex()
                                    .items_center()
                                    .gap(px(4.0))
                                    .p(px(6.0))
                                    .rounded(px(6.0))
                                    .cursor_pointer()
                                    .bg(if always_on_top { rgba(0x00d99230) } else { rgba(0xffffff0a) })
                                    .text_color(if always_on_top { rgb(0x00d992) } else { rgb(0x8b949e) })
                                    .hover(move |s| s.bg(if always_on_top { rgba(0x00d99240) } else { rgba(0xffffff15) }))
                                    .on_click(move |_, _, cx| {
                                        cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                            let p = c.plugins.entry(plugin_id.to_string()).or_insert_with(|| widget_core::PluginConfig {
                                                x: 0.0, y: 0.0, width: 300.0, height: 300.0, always_on_top: false, mouse_passthrough: false, loaded: true, enabled: true
                                            });
                                            p.always_on_top = !always_on_top;
                                        });
                                        let hwnd = widget_core::get_plugin_hwnd(plugin_id);
                                        if hwnd != 0 {
                                            unsafe {
                                                use windows_sys::Win32::UI::WindowsAndMessaging::{
                                                    SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
                                                };
                                                let insert_after = if !always_on_top { HWND_TOPMOST } else { HWND_NOTOPMOST };
                                                SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                                            }
                                        }
                                        widget_core::save_config_now(cx);
                                        cx.refresh_windows();
                                    })
                                    .child(div().text_sm().font_weight(FontWeight::MEDIUM).child("📌 置顶"))
                            )
                            .child(
                                div()
                                    .id(SharedString::from(format!("{}-ghost", plugin_id)))
                                    .flex()
                                    .items_center()
                                    .gap(px(4.0))
                                    .p(px(6.0))
                                    .rounded(px(6.0))
                                    .cursor_pointer()
                                    .bg(if mouse_passthrough { rgba(0x00d99230) } else { rgba(0xffffff0a) })
                                    .text_color(if mouse_passthrough { rgb(0x00d992) } else { rgb(0x8b949e) })
                                    .hover(move |s| s.bg(if mouse_passthrough { rgba(0x00d99240) } else { rgba(0xffffff15) }))
                                    .on_click(move |_, _, cx| {
                                        cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                            let p = c.plugins.entry(plugin_id.to_string()).or_insert_with(|| widget_core::PluginConfig {
                                                x: 0.0, y: 0.0, width: 300.0, height: 300.0, always_on_top: false, mouse_passthrough: false, loaded: true, enabled: true
                                            });
                                            p.mouse_passthrough = !mouse_passthrough;
                                        });
                                        let hwnd = widget_core::get_plugin_hwnd(plugin_id);
                                        if hwnd != 0 {
                                            unsafe {
                                                use windows_sys::Win32::UI::WindowsAndMessaging::{
                                                    GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_TRANSPARENT, WS_EX_LAYERED,
                                                };
                                                let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                                                SetWindowLongW(
                                                    hwnd,
                                                    GWL_EXSTYLE,
                                                    if !mouse_passthrough {
                                                        style | WS_EX_TRANSPARENT as i32 | WS_EX_LAYERED as i32
                                                    } else {
                                                        style & !(WS_EX_TRANSPARENT as i32 | WS_EX_LAYERED as i32)
                                                    },
                                                );
                                            }
                                        }
                                        widget_core::save_config_now(cx);
                                        cx.refresh_windows();
                                    })
                                    .child(div().text_sm().font_weight(FontWeight::MEDIUM).child("👻 穿透"))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .child(
                                Button::new(("btn-settings", kind as usize), "")
                                    .variant(ButtonVariant::Ghost)
                                    .icon(gpui_component::IconName::Settings)
                                    .on_click(move |_, _, cx| {
                                        let cb = cx.try_global::<widget_core::OpenPluginSettingsCallback>().cloned();
                                        if let Some(cb) = cb {
                                            cb.0(cx, plugin_id);
                                        }
                                    }),
                            )
                            .child(
                                Button::new(("btn-load", kind as usize), load_label)
                                    .variant(ButtonVariant::Outline)
                                    .on_click(move |_, _, cx| {
                                        let next_loaded = !cx
                                            .try_global::<widget_core::UIState>()
                                            .is_none_or(|s| s.is_plugin_loaded(plugin_id));
                                        cx.update_global::<widget_core::UIState, _>(|s, _| {
                                            s.plugin_loaded.insert(plugin_id.to_string(), next_loaded);
                                            if !next_loaded {
                                                s.plugin_enabled.insert(plugin_id.to_string(), false);
                                            } else {
                                                s.plugin_enabled.insert(plugin_id.to_string(), true);
                                            }
                                        });

                                        cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                            let cfg = c.plugins.entry(plugin_id.to_string()).or_insert_with(|| widget_core::PluginConfig {
                                                x: 0.0,
                                                y: 0.0,
                                                width: 0.0,
                                                height: 0.0,
                                                always_on_top: false,
                                                mouse_passthrough: false,
                                                loaded: next_loaded,
                                                enabled: next_loaded,
                                            });
                                            cfg.loaded = next_loaded;
                                            if !next_loaded {
                                                cfg.enabled = false;
                                            } else {
                                                cfg.enabled = true;
                                            }
                                        });
                                        widget_core::save_config_now(cx);

                                        let plugin_id_string = plugin_id.to_string();
                                        if let Some(cb) = cx.try_global::<widget_core::TogglePluginCallback>().cloned() {
                                            cx.defer(move |cx| {
                                                (cb.0)(cx, &plugin_id_string, next_loaded);
                                            });
                                        }

                                        cx.refresh_windows();
                                    }),
                            )
                            .child(
                                Button::new(("btn-enable", kind as usize), enable_label)
                                    .variant(if is_enabled { ButtonVariant::Secondary } else { ButtonVariant::Default })
                                    .on_click(move |_, _, cx| {
                                        // 如果未加载，不允许点击启用
                                        let is_loaded = cx.try_global::<widget_core::UIState>().is_none_or(|s| s.is_plugin_loaded(plugin_id));
                                        if !is_loaded {
                                            return;
                                        }
                                        let next_enabled = !cx
                                            .try_global::<widget_core::UIState>()
                                            .is_none_or(|s| s.is_plugin_enabled(plugin_id));
                                        cx.update_global::<widget_core::UIState, _>(|s, _| {
                                            s.plugin_enabled.insert(plugin_id.to_string(), next_enabled);
                                        });

                                        cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                            if let Some(cfg) = c.plugins.get_mut(plugin_id) {
                                                cfg.enabled = next_enabled;
                                            }
                                        });
                                        widget_core::save_config_now(cx);

                                        let hwnd = widget_core::get_plugin_hwnd(plugin_id);
                                        if hwnd != 0 {
                                            unsafe {
                                                if next_enabled {
                                                    windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW);
                                                } else {
                                                    windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE);
                                                }
                                            }
                                        }
                                        cx.refresh_windows();
                                    }),
                            ),
                    )
            )
    }

    fn render_widgets_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let plugin_list = cx
            .try_global::<widget_core::PluginList>()
            .map(|list| list.0.clone())
            .unwrap_or_default();

        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .p(px(24.0))
            .gap(px(16.0))
            .overflow_hidden()
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
                                    .child("小部件库 (市场)"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(rgb(0xb8b3b0))
                                    .child("发现并安装社区开发的桌面功能扩展"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .w_full()
                    .gap(px(16.0))
                    .flex_wrap()
                    .pb(px(24.0))
                    .children(plugin_list.into_iter().map(|meta| {
                        let is_loaded = cx
                            .try_global::<widget_core::UIState>()
                            .is_none_or(|s| s.is_plugin_loaded(meta.id));
                        self.market_plugin_card(
                            meta.name,
                            meta.id,
                            meta.description,
                            meta.icon,
                            meta.version,
                            meta.author,
                            is_loaded,
                        )
                    })),
            )
    }

    #[allow(clippy::too_many_arguments)]
    fn market_plugin_card(
        &self,
        name: &'static str,
        id_str: &'static str,
        desc: &'static str,
        icon: gpui_component::IconName,
        version: &'static str,
        author: &'static str,
        is_loaded: bool,
    ) -> impl IntoElement {
        use crate::components::button::{Button, ButtonVariant};
        div()
            .flex()
            .flex_col()
            .w(px(320.0))
            .p(px(20.0))
            .gap(px(16.0))
            .bg(rgb(0x101010))
            .border_1()
            .border_color(rgb(0x3d3a39))
            .rounded(px(8.0))
            .hover(|s| s.border_color(rgba(0x00d99280)))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(12.0))
                    .child(
                        div()
                            .w(px(40.0))
                            .h(px(40.0))
                            .rounded(px(8.0))
                            .bg(rgba(0xffffff0a))
                            .flex()
                            .justify_center()
                            .items_center()
                            .child(
                                div()
                                    .text_color(rgb(0xb8b3b0))
                                    .child(gpui_component::Icon::new(icon)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xf2f2f2))
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x8b949e))
                                    .child(id_str),
                            ),
                    ),
            )
            .child(
                div()
                    .h(px(48.0))
                    .text_sm()
                    .text_color(rgb(0xb8b3b0))
                    .child(desc),
            )
            .child(
                div()
                    .flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(div().text_xs().text_color(rgb(0x8b949e)).child(author))
                            .child(div().w(px(4.0)).h(px(4.0)).rounded_full().bg(rgb(0x3d3a39)))
                            .child(div().text_xs().text_color(rgb(0x8b949e)).child(version)),
                    )
                    .child(
                        Button::new(id_str, if is_loaded { "卸载" } else { "获取/安装" })
                            .variant(if is_loaded {
                                ButtonVariant::Outline
                            } else {
                                ButtonVariant::Default
                            })
                            .on_click(move |_, _, cx| {
                                let next_loaded = !is_loaded;
                                cx.update_global::<widget_core::UIState, _>(|s, _| {
                                    s.plugin_loaded.insert(id_str.to_string(), next_loaded);
                                    if !next_loaded {
                                        s.plugin_enabled.insert(id_str.to_string(), false);
                                    } else {
                                        s.plugin_enabled.insert(id_str.to_string(), true);
                                    }
                                });

                                cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                                    let cfg =
                                        c.plugins.entry(id_str.to_string()).or_insert_with(|| {
                                            widget_core::PluginConfig {
                                                x: 0.0,
                                                y: 0.0,
                                                width: 300.0,
                                                height: 300.0,
                                                always_on_top: false,
                                                mouse_passthrough: false,
                                                loaded: next_loaded,
                                                enabled: next_loaded,
                                            }
                                        });
                                    cfg.loaded = next_loaded;
                                    if !next_loaded {
                                        cfg.enabled = false;
                                    } else {
                                        cfg.enabled = true;
                                    }
                                });
                                widget_core::save_config_now(cx);

                                let id_string = id_str.to_string();
                                if let Some(cb) = cx
                                    .try_global::<widget_core::TogglePluginCallback>()
                                    .cloned()
                                {
                                    cx.defer(move |cx| {
                                        (cb.0)(cx, &id_string, next_loaded);
                                    });
                                }

                                cx.refresh_windows();
                            }),
                    ),
            )
    }

    fn render_settings_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let auto_start = cx
            .try_global::<widget_core::AppConfig>()
            .is_some_and(|c| c.auto_start);

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
                "开机自启动",
                "系统启动时自动运行应用",
                "auto-start",
                auto_start,
                move |val, cx| {
                    cx.update_global::<widget_core::AppConfig, _>(|c, _| {
                        c.auto_start = val;
                    });
                    if let Ok(exe_path) = std::env::current_exe() {
                        if let Some(exe_str) = exe_path.to_str() {
                            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                            if let Ok(run_key) = hkcu.open_subkey_with_flags(
                                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                                winreg::enums::KEY_ALL_ACCESS,
                            ) {
                                if val {
                                    let exe_path_quoted = format!("\"{}\"", exe_str);
                                    let _ = run_key.set_value("WidgetRS", &exe_path_quoted);
                                } else {
                                    let _ = run_key.delete_value("WidgetRS");
                                }
                                let _ = run_key.delete_value("Widget RS");
                            }
                        }
                    }
                    widget_core::save_config_now(cx);
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
