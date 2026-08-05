use crate::model::{StretchlyConfig, StretchlyStats};
use gpui::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use widget_core::AppConfig;

/// 设置页读取统计数据所用的全局信号
#[derive(Clone)]
pub struct StretchlyLiveStats(pub StretchlyStats);
impl Global for StretchlyLiveStats {}

pub struct StretchlySettingsView {
    pub config: StretchlyConfig,
    stats: StretchlyStats,
}

impl StretchlySettingsView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx
            .try_global::<AppConfig>()
            .and_then(|cfg| cfg.get_plugin_data::<StretchlyConfig>("stretchly_widget"))
            .unwrap_or_default();
        let stats = cx
            .try_global::<StretchlyLiveStats>()
            .map(|s| s.0.clone())
            .unwrap_or_default();
        Self { config, stats }
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        cx.update_global::<AppConfig, _>(|cfg, _| {
            cfg.set_plugin_data("stretchly_widget", &self.config);
        });
        widget_core::save_config_now(cx);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_number_input(
        &self,
        id_prefix: &'static str,
        label: &str,
        unit_desc: &str,
        value: u64,
        multiplier: u64,
        min_display: u64,
        max_display: u64,
        step_display: u64,
        cx: &mut Context<Self>,
        on_change: impl Fn(&mut Self, u64, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let display_val = value / multiplier;
        let on_minus = std::rc::Rc::new(on_change);
        let on_plus = on_minus.clone();

        div()
            .flex()
            .justify_between()
            .items_center()
            .w_full()
            .px(px(12.0))
            .py(px(8.0))
            .rounded(px(8.0))
            .bg(rgba(0xffffff05))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xf2f2f2))
                            .child(label.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgba(0x94a3b870))
                            .child(unit_desc.to_string()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .flex_shrink_0()
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-minus", id_prefix)))
                            .w(px(26.0))
                            .h(px(26.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(5.0))
                            .bg(rgba(0xffffff10))
                            .hover(|s| s.bg(rgba(0xffffff22)))
                            .cursor_pointer()
                            .child(div().text_sm().text_color(rgb(0xffffff)).child("-"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if display_val > min_display {
                                    let v =
                                        display_val.saturating_sub(step_display).max(min_display);
                                    on_minus(this, v * multiplier, cx);
                                }
                            })),
                    )
                    .child(
                        div()
                            .w(px(36.0))
                            .flex()
                            .justify_center()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x00d992))
                            .child(display_val.to_string()),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-plus", id_prefix)))
                            .w(px(26.0))
                            .h(px(26.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(5.0))
                            .bg(rgba(0xffffff10))
                            .hover(|s| s.bg(rgba(0xffffff22)))
                            .cursor_pointer()
                            .child(div().text_sm().text_color(rgb(0xffffff)).child("+"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if display_val < max_display {
                                    let v =
                                        display_val.saturating_add(step_display).min(max_display);
                                    on_plus(this, v * multiplier, cx);
                                }
                            })),
                    ),
            )
    }

    fn section_header(label: &str) -> impl IntoElement {
        div()
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(rgba(0x94a3b899))
            .pt(px(10.0))
            .pb(px(4.0))
            .child(label.to_string())
    }

    fn stat_chip(label: &str, value: u32, color: Rgba) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .child(
                div()
                    .text_xs()
                    .text_color(rgba(0x94a3b880u32))
                    .child(label.to_string()),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(color)
                    .child(value.to_string()),
            )
    }
}

impl Render for StretchlySettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 提前克隆所有需要渲染的値
        let mini_break_interval = self.config.mini_break_interval;
        let mini_break_duration = self.config.mini_break_duration;
        let long_break_interval = self.config.long_break_interval;
        let long_break_duration = self.config.long_break_duration;
        let warning_seconds = self.config.warning_seconds;
        let skip_delay_seconds = self.config.skip_delay_seconds;
        let postpone_minutes = self.config.postpone_minutes;
        // P3: 统计快照
        if let Some(live) = cx.try_global::<StretchlyLiveStats>() {
            self.stats = live.0.clone();
        }
        let stats = self.stats.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x0d1117))
            .border_1()
            .border_color(rgb(0x30363d))
            // ── 标题栏 ───────────────────────────────────────────────────────
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .w_full()
                    .h(px(46.0))
                    .flex_shrink_0()
                    .bg(rgb(0x161b22))
                    .border_b_1()
                    .border_color(rgb(0x30363d))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .items_center()
                            .h_full()
                            .pl(px(16.0))
                            .id("titlebar-drag")
                            .on_mouse_down(MouseButton::Left, |_, win, _| {
                                if let Ok(h) = win.window_handle() {
                                    if let RawWindowHandle::Win32(h) = h.as_raw() {
                                        unsafe {
                                            windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                                            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                                h.hwnd.get(),
                                                windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN,
                                                windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION
                                                    as usize,
                                                0,
                                            );
                                        }
                                    }
                                }
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xe6edf3))
                                    .child("休息提醒 - 设置"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(46.0))
                            .h_full()
                            .hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                            .text_color(rgb(0x8b949e))
                            .cursor_pointer()
                            .id("close-btn")
                            .on_click(|_, win, _| {
                                if let Ok(h) = win.window_handle() {
                                    if let RawWindowHandle::Win32(h) = h.as_raw() {
                                        unsafe {
                                            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                                h.hwnd.get(),
                                                windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                                                0,
                                                0,
                                            );
                                        }
                                    }
                                }
                            })
                            .child(gpui_component::Icon::new(gpui_component::IconName::Close)),
                    ),
            )
            // ── 内容区（单一纵向列，不使用 scroll） ─────────────────────────
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p(px(16.0))
                    .gap(px(6.0))
                    // 工作节奏
                    .child(Self::section_header("工作节奏"))
                    .child(self.render_number_input(
                        "mini-int",
                        "微休间隔",
                        "分钟",
                        mini_break_interval,
                        60,
                        1, 60, 1,
                        cx,
                        |this, val, cx| {
                            this.config.mini_break_interval = val;
                            cx.notify();
                        },
                    ))
                    .child(self.render_number_input(
                        "mini-dur",
                        "微休时长",
                        "秒",
                        mini_break_duration,
                        1,
                        5, 120, 5,
                        cx,
                        |this, val, cx| {
                            this.config.mini_break_duration = val;
                            cx.notify();
                        },
                    ))
                    .child(self.render_number_input(
                        "long-int",
                        "长休间隔",
                        "分钟",
                        long_break_interval,
                        60,
                        10, 240, 10,
                        cx,
                        |this, val, cx| {
                            this.config.long_break_interval = val;
                            cx.notify();
                        },
                    ))
                    .child(self.render_number_input(
                        "long-dur",
                        "长休时长",
                        "分钟",
                        long_break_duration,
                        60,
                        1, 60, 1,
                        cx,
                        |this, val, cx| {
                            this.config.long_break_duration = val;
                            cx.notify();
                        },
                    ))
                    // 交互行为
                    .child(Self::section_header("交互行为"))
                    .child(self.render_number_input(
                        "warning",
                        "预警时间",
                        "秒（休息前多少秒显示预警）",
                        warning_seconds,
                        1,
                        0, 120, 10,
                        cx,
                        |this, val, cx| {
                            this.config.warning_seconds = val;
                            cx.notify();
                        },
                    ))
                    .child(self.render_number_input(
                        "skip-delay",
                        "跳过保护",
                        "秒（休息开始后多少秒内禁止跳过）",
                        skip_delay_seconds,
                        1,
                        0, 30, 1,
                        cx,
                        |this, val, cx| {
                            this.config.skip_delay_seconds = val;
                            cx.notify();
                        },
                    ))
                    .child(self.render_number_input(
                        "postpone",
                        "推迟时长",
                        "分钟（点击推迟后延后多久）",
                        postpone_minutes,
                        1,
                        1, 30, 1,
                        cx,
                        |this, val, cx| {
                            this.config.postpone_minutes = val;
                            cx.notify();
                        },
                    ))
                    // 今日统计卡片
                    .child(Self::section_header("今日统计"))
                    .child(
                        div()
                            .w_full()
                            .p(px(12.0))
                            .rounded(px(8.0))
                            .bg(rgba(0xffffff05))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            // 第一行：休息数据
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .flex()
                                            .gap(px(16.0))
                                            .child(Self::stat_chip("微休", stats.mini_breaks_done, rgb(0x00d992)))
                                            .child(Self::stat_chip("长休", stats.long_breaks_done, rgb(0x38bdf8)))
                                            .child(Self::stat_chip("跳过", stats.breaks_skipped, rgb(0xf87171)))
                                            .child(Self::stat_chip("推迟", stats.breaks_postponed, rgb(0xfbbf24))),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_end()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0x00d992))
                                                    .child(format!("{} 分钟", stats.focus_minutes)),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgba(0x94a3b870))
                                                    .child("专注时长"),
                                            ),
                                    ),
                            )
                            // 第二行：日期 + 重置按鈕
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgba(0x94a3b850))
                                            .child(if stats.date.is_empty() {
                                                "今天".to_string()
                                            } else {
                                                stats.date.clone()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .id("reset-stats-btn")
                                            .px(px(10.0))
                                            .py(px(3.0))
                                            .rounded(px(4.0))
                                            .border_1()
                                            .border_color(rgba(0xffffff15))
                                            .bg(rgba(0xffffff08))
                                            .hover(|s| s.bg(rgba(0xf8717130)))
                                            .cursor_pointer()
                                            .text_xs()
                                            .text_color(rgba(0xf8717180))
                                            .child("重置")
                                            .on_click(cx.listener(|_this, _, _, cx| {
                                                cx.set_global(StretchlyLiveStats(
                                                    StretchlyStats::default(),
                                                ));
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    // -- 按钮区
                    .child(
                        div()
                            .mt(px(14.0))
                            .pt(px(14.0))
                            .border_t_1()
                            .border_color(rgb(0x21262d))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .id("save-btn")
                                    .w_full()
                                    .py(px(10.0))
                                    .flex()
                                    .justify_center()
                                    .rounded(px(7.0))
                                    .bg(rgb(0x00d992))
                                    .hover(|s| s.bg(rgba(0x00d992cc)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, win, cx| {
                                        this.save(cx);
                                        if let Ok(h) = win.window_handle() {
                                            if let RawWindowHandle::Win32(h) = h.as_raw() {
                                                unsafe {
                                                    windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                                        h.hwnd.get(),
                                                        windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                                                        0, 0,
                                                    );
                                                }
                                            }
                                        }
                                    }))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x050507))
                                            .child("保存并关闭"),
                                    ),
                            )
                            .child(
                                div()
                                    .id("apply-now-btn")
                                    .w_full()
                                    .py(px(9.0))
                                    .flex()
                                    .justify_center()
                                    .rounded(px(7.0))
                                    .border_1()
                                    .border_color(rgba(0xf59e0b60))
                                    .bg(rgba(0xf59e0b0d))
                                    .hover(|s| s.bg(rgba(0xf59e0b20)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, win, cx| {
                                        this.save(cx);
                                        cx.set_global(crate::StretchlyApplyNow(true));
                                        if let Ok(h) = win.window_handle() {
                                            if let RawWindowHandle::Win32(h) = h.as_raw() {
                                                unsafe {
                                                    windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                                        h.hwnd.get(),
                                                        windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                                                        0, 0,
                                                    );
                                                }
                                            }
                                        }
                                    }))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xf59e0b))
                                            .child("立即生效（重置计时器）"),
                                    ),
                            ),
                    ),
            )
    }
}
