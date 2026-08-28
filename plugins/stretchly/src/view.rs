use gpui::prelude::FluentBuilder;
use gpui::*;
use raw_window_handle::HasWindowHandle;
use std::time::{Duration, Instant};

use crate::model::{BreakState, StretchlyConfig, StretchlyModel};
use crate::tips::current_tip;
use widget_core::AppConfig;

use crate::overlay::BreakOverlay;

/// 枚举所有显示器的物理坐标，返回 (x, y, w, h, is_primary)
/// is_primary = 该显示器是否是 widget_hwnd 所在的显示器
fn get_all_monitor_rects(widget_hwnd: isize) -> Vec<(i32, i32, i32, i32, bool)> {
    unsafe {
        use windows_sys::Win32::Foundation::{BOOL, RECT};
        use windows_sys::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MONITORINFO,
        };

        let _ = widget_hwnd; // widget_hwnd is no longer needed since we detect primary differently, but we keep the parameter.

        struct State {
            rects: Vec<(i32, i32, i32, i32, bool)>,
        }
        let mut state = State { rects: Vec::new() };

        unsafe extern "system" fn cb(
            hmon: isize,
            _hdc: isize,
            _lp_rect: *mut RECT,
            lparam: isize,
        ) -> BOOL {
            let s = &mut *(lparam as *mut State);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(hmon, &mut info as *mut _);
            let r = info.rcMonitor;
            let is_primary = (info.dwFlags & 1) != 0; // MONITORINFOF_PRIMARY = 1
            s.rects.push((
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                is_primary,
            ));
            1
        }

        EnumDisplayMonitors(
            0,
            std::ptr::null(),
            Some(cb),
            &mut state as *mut State as isize,
        );
        state.rects
    }
}

pub struct StretchlyWidget {
    was_working: bool,
    /// P2: 追踪预警状态变化，用于触发一次性通知
    prev_warning: bool,
    model: StretchlyModel,
    /// 当前休息阶段的开始时刻（用于计算跳过延迟）
    break_started_at: Option<Instant>,
    /// 缓存最后一次从 render() 取到的 HWND，供 timer 使用
    cached_hwnd: isize,
    _timer: gpui::Task<()>,
}

impl StretchlyWidget {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx
            .try_global::<AppConfig>()
            .and_then(|cfg| cfg.get_plugin_data::<StretchlyConfig>("stretchly_widget"));

        let model = StretchlyModel::new(config);

        let this = cx.weak_entity();
        let app_cx: &mut App = cx;
        let _timer = app_cx.spawn(async move |async_cx| {
            loop {
                async_cx
                    .background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                let res = async_cx.update(|cx| {
                    let _ = this.update(cx, |this, cx| {
                        // ── 处理 BreakOverlay 按钮回调请求 ────────────────────────────────
                        if let Some(req) = cx
                            .try_global::<crate::StretchlyOverlayRequest>()
                            .and_then(|r| r.0.clone())
                        {
                            // 先清除请求，再执行，避免重入
                            cx.set_global(crate::StretchlyOverlayRequest(None));
                            match req {
                                crate::StretchlyOverlayAction::Skip => this.model.skip(),
                                crate::StretchlyOverlayAction::Postpone => {
                                    this.model.skip_and_postpone()
                                }
                            }
                        }
                        // 热更新配置：排队，在下次状态切换时才真正生效
                        if let Some(cfg) = cx.try_global::<AppConfig>() {
                            if let Some(new_cfg) =
                                cfg.get_plugin_data::<StretchlyConfig>("stretchly_widget")
                            {
                                let apply_now = cx
                                    .try_global::<crate::StretchlyApplyNow>()
                                    .is_some_and(|s| s.0);
                                if apply_now {
                                    this.model.apply_config_now(new_cfg);
                                    cx.set_global(crate::StretchlyApplyNow(false));
                                } else {
                                    this.model.queue_config_update(new_cfg);
                                }
                            }
                        }
                        this.model.tick();
                        // P3: 将最新统计同步到全局，供设置页读取
                        cx.set_global(crate::StretchlyLiveStats(this.model.stats.clone()));

                        // ── 先发布快照，确保 BreakOverlay 首次渲染即有数据 ──────────────────────
                        if this.model.is_on_break() {
                            let remaining = this.model.time_remaining();
                            let rem_mins = remaining.as_secs() / 60;
                            let rem_secs = remaining.as_secs() % 60;
                            let time_str = format!("{:02}:{:02}", rem_mins, rem_secs);
                            let skip_delay = this.model.config.skip_delay_seconds;
                            let break_elapsed_secs = this
                                .break_started_at
                                .map(|t| t.elapsed().as_secs())
                                .unwrap_or(0);
                            let skip_available = break_elapsed_secs >= skip_delay;
                            let skip_countdown = skip_delay.saturating_sub(break_elapsed_secs);
                            let (is_mini, break_label, break_duration_label) = match this
                                .model
                                .state
                            {
                                BreakState::MiniBreak => (
                                    true,
                                    "微休",
                                    format!("{} 秒", this.model.config.mini_break_duration),
                                ),
                                BreakState::LongBreak => (
                                    false,
                                    "长休",
                                    format!("{} 分钟", this.model.config.long_break_duration / 60),
                                ),
                                BreakState::Working => (true, "", String::new()),
                            };
                            cx.set_global(crate::StretchlyBreakSnapshot {
                                state: this.model.state,
                                time_str,
                                progress: this.model.progress(),
                                break_label,
                                break_duration_label,
                                is_mini,
                                skip_available,
                                skip_label: if skip_available {
                                    "结束休息".to_string()
                                } else {
                                    format!("结束休息 ({}s)", skip_countdown)
                                },
                                postpone_mins: this.model.config.postpone_minutes,
                                tip: current_tip().to_string(),
                                allow_skip: this.model.config.allow_skip,
                                allow_postpone: this.model.config.allow_postpone,
                            });
                        }

                        // ── 状态切换：在 tick 回调里执行，避免在 render() 里调用 cx.open_window ──
                        let is_on_break = this.model.is_on_break();
                        if this.was_working != !is_on_break {
                            this.was_working = !is_on_break;
                            let hwnd = this.cached_hwnd;
                            if is_on_break {
                                cx.set_global(crate::StretchlyBreakActive(true));
                                this.break_started_at = Some(Instant::now());
                                // 取消小组件置顶，使其位于全屏遮罩下方
                                if hwnd != 0 {
                                    unsafe {
                                        use windows_sys::Win32::UI::WindowsAndMessaging::{
                                            SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE,
                                            SWP_NOSIZE,
                                        };
                                        SetWindowPos(
                                            hwnd,
                                            HWND_BOTTOM,
                                            0,
                                            0,
                                            0,
                                            0,
                                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                                        );
                                    }
                                }
                                // 为每块显示器创建独立的 BreakOverlay 窗口
                                for (x, y, w, h, is_primary) in get_all_monitor_rects(hwnd) {
                                    let _ = cx.open_window(
                                        WindowOptions {
                                            titlebar: None,
                                            window_background:
                                                WindowBackgroundAppearance::Transparent,
                                            kind: WindowKind::PopUp,
                                            is_resizable: false,
                                            // 我们依然用原生的 bounds 去初始化，只不过随后我们的 hook 会强制接管
                                            window_bounds: Some(WindowBounds::Windowed(
                                                Bounds::new(
                                                    Point::new(px(x as f32), px(y as f32)),
                                                    size(px(w as f32), px(h as f32)),
                                                ),
                                            )),
                                            ..Default::default()
                                        },
                                        move |_window, cx| {
                                            cx.new(|cx| {
                                                let sub1 = cx.observe_global::<crate::StretchlyBreakSnapshot>(
                                                    |_, cx| cx.notify(),
                                                );
                                                let sub2 = cx.observe_global::<crate::StretchlyBreakActive>(
                                                    |_, cx| cx.notify(),
                                                );
                                                BreakOverlay::new(is_primary, (x, y, w, h), vec![sub1, sub2])
                                            })
                                        },
                                    );
                                }
                            } else {
                                cx.set_global(crate::StretchlyBreakActive(false));
                                this.break_started_at = None;
                                if hwnd != 0 {
                                    unsafe {
                                        use windows_sys::Win32::UI::WindowsAndMessaging::{
                                            SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
                                            SWP_NOSIZE,
                                        };
                                        SetWindowPos(
                                            hwnd,
                                            HWND_TOPMOST,
                                            0,
                                            0,
                                            0,
                                            0,
                                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                                        );
                                    }
                                }
                            }
                        }

                        cx.notify();
                    });
                });
                if res.is_err() {
                    break;
                }
            }
        });

        Self {
            was_working: true,
            prev_warning: false,
            model,
            break_started_at: None,
            cached_hwnd: 0,
            _timer,
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

    /// 休息中不显示拖拽条
    fn show_drag_handle(&self) -> bool {
        !self.model.is_on_break()
    }
}

impl Render for StretchlyWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ── 缓存 HWND 供 timer 回调使用 ─────────────────────────────────────
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

        // ── 状态切换逻辑已移至 timer tick，render() 仅读取状态 ───────────────
        let is_on_break = self.model.is_on_break();

        // ── 预计算渲染数据 ────────────────────────────────────────────────────
        let remaining = self.model.time_remaining();
        let progress = self.model.progress();
        let is_warning = self.model.is_warning();
        let is_paused = self.model.is_paused;
        let mini_taken = self.model.mini_breaks_taken;
        let mini_total = self.model.mini_breaks_in_cycle();
        // P3: 今日统计摘要
        let stats_mini = self.model.stats.mini_breaks_done;
        let stats_long = self.model.stats.long_breaks_done;
        let stats_skip = self.model.stats.breaks_skipped;
        let stats_focus = self.model.stats.focus_minutes;

        let allow_skip = self.model.config.allow_skip;
        let allow_postpone = self.model.config.allow_postpone;

        // P2: 预警开始时触发一次性系统通知
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

        // ══════════════════════════════════════════════════════════════════════
        // 工作中/休息中：紧凑小组件（休息时作为底层显示）
        // ══════════════════════════════════════════════════════════════════════

        let bg_color = if is_warning {
            rgba(0x1a0e04efu32)
        } else {
            rgba(0x0d111aefu32)
        };

        let progress_color = if is_warning {
            rgb(0xf59e0bu32)
        } else {
            rgb(0x34d399u32)
        };
        let dot_color = if is_paused {
            rgb(0x6b7280u32)
        } else if is_warning {
            rgb(0xf59e0bu32)
        } else {
            rgb(0x34d399u32)
        };
        let status_label = if is_paused {
            "已暂停"
        } else if is_warning {
            "即将休息"
        } else {
            "专注中"
        };

        let dots: Vec<bool> = (0..mini_total).map(|i| i < mini_taken).collect();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .bg(bg_color)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .px(px(12.0))
                    .pt(px(10.0))
                    .pb(px(8.0))
                    .gap(px(6.0))
                    // ── 顶部行 ────────────────────────────────────────────────
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            // 左：圆点 + 状态 + 微休进度点
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .w(px(7.0))
                                            .h(px(7.0))
                                            .rounded(px(4.0))
                                            .bg(dot_color)
                                            .flex_shrink_0(),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xe2e8f0u32))
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
                                                    div().w(px(5.0)).h(px(5.0)).rounded(px(3.0)).bg(
                                                        if done {
                                                            rgba(0x34d39999u32)
                                                        } else {
                                                            rgba(0xffffff18u32)
                                                        },
                                                    )
                                                })),
                                        )
                                    }),
                            )
                            // 右：时间 + 按钮
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgba(0x94a3b8a0u32))
                                            .when(is_warning, |d| {
                                                d.text_color(rgba(0xf59e0be0u32))
                                                    .font_weight(FontWeight::MEDIUM)
                                            })
                                            .child(time_str),
                                    )
                                    // 暂停/继续
                                    .child(
                                        div()
                                            .px(px(6.0))
                                            .py(px(3.0))
                                            .rounded(px(5.0))
                                            .bg(rgba(0xffffff0du32))
                                            .hover(|s| s.bg(rgba(0xffffff1au32)))
                                            .cursor_pointer()
                                            .text_xs()
                                            .text_color(rgba(0x94a3b8ccu32))
                                            .id("pause-btn")
                                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                                this.model.toggle_pause();
                                                cx.notify();
                                            }))
                                            .child(if is_paused { "继续" } else { "暂停" }),
                                    )
                                    // 预警时：推迟按钮
                                    .when(is_warning && allow_postpone, |d| {
                                        d.child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(3.0))
                                                .rounded(px(5.0))
                                                .bg(rgba(0xf59e0b18u32))
                                                .hover(|s| s.bg(rgba(0xf59e0b28u32)))
                                                .cursor_pointer()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xf59e0bu32))
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
                                    // 正常工作时：立即休息按钮（当作跳过当前工作阶段，与 allow_skip 关联或总是允许？这里受 allow_skip 限制）
                                    .when(!is_warning && !is_paused && allow_skip, |d| {
                                        d.child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(3.0))
                                                .rounded(px(5.0))
                                                .bg(rgba(0xffffff08u32))
                                                .hover(|s| s.bg(rgba(0xffffff15u32)))
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(rgba(0x94a3b880u32))
                                                .id("break-now-btn")
                                                .on_click(cx.listener(
                                                    |this, _: &ClickEvent, _, cx| {
                                                        this.model.skip();
                                                        cx.notify();
                                                    },
                                                ))
                                                .child("休息"),
                                        )
                                    }),
                            ),
                    )
                    // ── 进度条 ────────────────────────────────────────────────
                    .child(
                        div()
                            .w_full()
                            .h(px(3.0))
                            .rounded(px(2.0))
                            .bg(rgba(0xffffff0cu32))
                            .child(
                                div()
                                    .h_full()
                                    .rounded(px(2.0))
                                    .bg(progress_color)
                                    .w(relative(progress)),
                            ),
                    )
                    // ── P3: 今日统计摘要（极简，不喧宾夺主）────────────────
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .justify_between()
                            .items_center()
                            .pt(px(3.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgba(0xffffff28u32))
                                    .child(format!(
                                        "微休 {}  长休 {}  跳过 {}",
                                        stats_mini, stats_long, stats_skip
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgba(0xffffff1cu32))
                                    .child(format!("专注 {} 分", stats_focus)),
                            ),
                    ),
            )
    }
}
