use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{Icon, IconName};
use raw_window_handle::HasWindowHandle;
use std::time::Instant;

use crate::details::render_details_card;
use crate::model::{BreakState, StretchlyConfig, StretchlyModel};
use crate::timer::spawn_stretchly_timer;
use widget_core::AppConfig;

pub struct StretchlyWidget {
    was_working: bool,
    prev_warning: bool,
    model: StretchlyModel,
    break_started_at: Option<Instant>,
    cached_hwnd: isize,
    show_details: bool,
    _timer: gpui::Task<()>,
}

impl StretchlyWidget {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx
            .try_global::<AppConfig>()
            .and_then(|cfg| cfg.get_plugin_data::<StretchlyConfig>("stretchly_widget"));

        let model = StretchlyModel::new(config);
        let this = cx.weak_entity();
        let _timer = spawn_stretchly_timer(this, cx);

        Self {
            was_working: true,
            prev_warning: false,
            model,
            break_started_at: None,
            cached_hwnd: 0,
            show_details: false,
            _timer,
        }
    }

    pub fn model(&self) -> &StretchlyModel {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut StretchlyModel {
        &mut self.model
    }

    pub fn was_working(&self) -> bool {
        self.was_working
    }

    pub fn set_was_working(&mut self, val: bool) {
        self.was_working = val;
    }

    pub fn break_started_at(&self) -> Option<Instant> {
        self.break_started_at
    }

    pub fn set_break_started_at(&mut self, val: Option<Instant>) {
        self.break_started_at = val;
    }

    pub fn cached_hwnd(&self) -> isize {
        self.cached_hwnd
    }

    /// 展开/折叠健康建议时，根据内容动态自适应调节窗口高度
    fn update_window_height(&self, hwnd: isize) {
        if hwnd == 0 {
            return;
        }
        unsafe {
            use windows_sys::Win32::Foundation::RECT;
            use windows_sys::Win32::Graphics::Dwm::{
                DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS,
            };
            use windows_sys::Win32::Graphics::Gdi::{
                InvalidateRect, RedrawWindow, RDW_ALLCHILDREN, RDW_FRAME, RDW_INVALIDATE,
                RDW_UPDATENOW,
            };
            use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GetWindowRect, SetWindowPos, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
                SWP_NOZORDER,
            };

            let dpi = GetDpiForWindow(hwnd);
            let scale = if dpi == 0 { 1.0 } else { dpi as f32 / 96.0 };
            // 折叠状态为紧凑 78px，展开状态自适应为 172px
            let target_log_h = if self.show_details { 172.0 } else { 78.0 };
            let target_phys_h = (target_log_h * scale).round() as i32;

            let mut rect: RECT = std::mem::zeroed();
            let hr = DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS as u32,
                &mut rect as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            );
            let phys_w = if hr == 0 {
                rect.right - rect.left
            } else if GetWindowRect(hwnd, &mut rect) != 0 {
                rect.right - rect.left
            } else {
                (280.0 * scale).round() as i32
            };

            SetWindowPos(
                hwnd,
                0,
                0,
                0,
                phys_w,
                target_phys_h,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
            InvalidateRect(hwnd, std::ptr::null(), 1);
            RedrawWindow(
                hwnd,
                std::ptr::null(),
                0,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_FRAME | RDW_ALLCHILDREN,
            );
        }
    }

    /// P2: 预警通知 — 闪烁任务栏提醒用户即将休息
    fn trigger_warning_notification(hwnd: isize) {
        if hwnd == 0 {
            return;
        }
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                FlashWindowEx, FLASHWINFO, FLASHW_TIMERNOFG, FLASHW_TRAY,
            };
            let fi = FLASHWINFO {
                cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
                hwnd,
                dwFlags: FLASHW_TRAY | FLASHW_TIMERNOFG,
                uCount: 4,
                dwTimeout: 0,
            };
            FlashWindowEx(&fi);
        }
    }
}

impl widget_core::WidgetContent for StretchlyWidget {
    fn plugin_id(&self) -> &'static str {
        "stretchly_widget"
    }

    fn drag_label(&self) -> &'static str {
        "拖拽移动"
    }

    fn show_drag_handle(&self) -> bool {
        !self.model.is_on_break()
    }
}

impl Render for StretchlyWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let hwnd = window
            .window_handle()
            .ok()
            .and_then(|h| {
                if let raw_window_handle::RawWindowHandle::Win32(h) = h.as_raw() {
                    Some(h.hwnd.get())
                } else {
                    None
                }
            })
            .unwrap_or(0);
        if hwnd != 0 {
            self.cached_hwnd = hwnd;
        }

        let is_on_break = self.model.is_on_break();
        let remaining = self.model.time_remaining();
        let progress = self.model.progress();
        let is_warning = self.model.is_warning();
        let is_paused = self.model.is_paused;
        let mini_taken = self.model.mini_breaks_taken;
        let mini_total = self.model.mini_breaks_in_cycle();

        let stats_mini = self.model.stats.mini_breaks_done;
        let stats_long = self.model.stats.long_breaks_done;
        let stats_skip = self.model.stats.breaks_skipped;
        let stats_focus = self.model.stats.focus_minutes;

        let allow_skip = self.model.config.allow_skip;
        let allow_postpone = self.model.config.allow_postpone;

        if is_warning && !self.prev_warning {
            Self::trigger_warning_notification(hwnd);
        }
        self.prev_warning = is_warning;

        let rem_mins = remaining.as_secs() / 60;
        let rem_secs = remaining.as_secs() % 60;
        let time_str = if is_on_break {
            format!("{:02}:{:02}", rem_mins, rem_secs)
        } else if rem_mins >= 1 {
            format!("约 {} 分钟", rem_mins + if rem_secs >= 30 { 1 } else { 0 })
        } else {
            format!("{} 秒", rem_secs)
        };

        let (dot_color, progress_color, bg_color, status_label) = if is_paused {
            (rgb(0x94a3b8), rgb(0x64748b), rgba(0x0a1220db), "已暂停")
        } else if is_warning {
            (rgb(0xfb923c), rgb(0xfb923c), rgba(0x1a0e04eb), "即将休息")
        } else if is_on_break {
            if matches!(self.model.state, BreakState::MiniBreak) {
                (rgb(0x38bdf8), rgb(0x38bdf8), rgba(0x06182deb), "微休息中")
            } else {
                (rgb(0xa78bfa), rgb(0xa78bfa), rgba(0x160c2eeb), "长休息中")
            }
        } else {
            (rgb(0x34d399), rgb(0x34d399), rgba(0x0a1220db), "专注中")
        };

        let show_details = self.show_details;
        let dots: Vec<bool> = (0..mini_total).map(|i| i < mini_taken).collect();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .bg(bg_color)
            .rounded(px(14.0))
            .border_1()
            .border_color(rgba(0xffffff20))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .px(px(12.0))
                    .py(px(10.0))
                    .gap(px(6.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .w(px(8.0))
                                            .h(px(8.0))
                                            .rounded_full()
                                            .bg(dot_color)
                                            .flex_shrink_0(),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xf8fafc))
                                            .child(status_label),
                                    )
                                    .when(!is_warning && !is_paused && mini_total > 1, |d| {
                                        d.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(3.0))
                                                .ml(px(2.0))
                                                .children(dots.iter().map(|&done| {
                                                    div().w(px(5.0)).h(px(5.0)).rounded_full().bg(
                                                        if done {
                                                            rgb(0x34d399)
                                                        } else {
                                                            rgba(0xffffff20)
                                                        },
                                                    )
                                                })),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if is_warning {
                                                rgb(0xfb923c)
                                            } else {
                                                rgba(0xffffffaa)
                                            })
                                            .font_weight(FontWeight::MEDIUM)
                                            .child(time_str),
                                    )
                                    .child(
                                        div()
                                            .px(px(6.0))
                                            .py(px(2.5))
                                            .rounded(px(4.0))
                                            .bg(rgba(0xffffff10))
                                            .hover(|s| s.bg(rgba(0xffffff20)))
                                            .cursor_pointer()
                                            .text_xs()
                                            .text_color(rgb(0xf1f5f9))
                                            .id("pause-btn")
                                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                                this.model.toggle_pause();
                                                cx.notify();
                                            }))
                                            .child(if is_paused { "继续" } else { "暂停" }),
                                    )
                                    .when(is_warning && allow_postpone, |d| {
                                        d.child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(2.5))
                                                .rounded(px(4.0))
                                                .bg(rgba(0xfb923c25))
                                                .hover(|s| s.bg(rgba(0xfb923c40)))
                                                .cursor_pointer()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xfb923c))
                                                .id("postpone-warning-btn")
                                                .on_click(cx.listener(
                                                    |this, _: &ClickEvent, _, cx| {
                                                        this.model.postpone();
                                                        cx.notify();
                                                    },
                                                ))
                                                .child("推迟"),
                                        )
                                    })
                                    .when(
                                        !is_warning && !is_paused && !is_on_break && allow_skip,
                                        |d| {
                                            d.child(
                                                div()
                                                    .px(px(6.0))
                                                    .py(px(2.5))
                                                    .rounded(px(4.0))
                                                    .bg(rgba(0x38bdf818))
                                                    .hover(|s| s.bg(rgba(0x38bdf830)))
                                                    .cursor_pointer()
                                                    .text_xs()
                                                    .text_color(rgb(0x38bdf8))
                                                    .id("break-now-btn")
                                                    .on_click(cx.listener(
                                                        |this, _: &ClickEvent, _, cx| {
                                                            this.model.skip();
                                                            cx.notify();
                                                        },
                                                    ))
                                                    .child("休息"),
                                            )
                                        },
                                    )
                                    .child(
                                        div()
                                            .w(px(24.0))
                                            .h(px(22.0))
                                            .flex()
                                            .justify_center()
                                            .items_center()
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .text_color(rgba(0xffffffaa))
                                            .hover(|s| {
                                                s.bg(rgba(0xffffff20)).text_color(rgb(0xffffff))
                                            })
                                            .id("stretchly-expand-btn")
                                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                                this.show_details = !this.show_details;
                                                let hwnd = window
                                                    .window_handle()
                                                    .ok()
                                                    .and_then(|h| {
                                                        if let raw_window_handle::RawWindowHandle::Win32(h) = h.as_raw() {
                                                            Some(h.hwnd.get())
                                                        } else {
                                                            None
                                                        }
                                                    })
                                                    .unwrap_or(this.cached_hwnd);
                                                this.update_window_height(hwnd);
                                                cx.notify();
                                                cx.refresh_windows();
                                            }))
                                            .child(
                                                Icon::new(if show_details {
                                                    IconName::ChevronUp
                                                } else {
                                                    IconName::ChevronDown
                                                })
                                                .size(px(12.0)),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(px(3.5))
                            .rounded_full()
                            .bg(rgba(0xffffff12))
                            .child(
                                div()
                                    .h_full()
                                    .rounded_full()
                                    .bg(progress_color)
                                    .w(relative(progress)),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .justify_between()
                            .items_center()
                            .pt(px(2.0))
                            .child(div().text_xs().text_color(rgba(0xffffff50)).child(format!(
                                "微休 {}  长休 {}  跳过 {}",
                                stats_mini, stats_long, stats_skip
                            )))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgba(0xffffff50))
                                    .child(format!("专注 {} 分钟", stats_focus)),
                            ),
                    )
                    .when(show_details, |container| {
                        container.child(render_details_card(
                            mini_taken,
                            mini_total,
                            dot_color,
                            is_warning,
                            is_paused,
                        ))
                    }),
            )
    }
}
