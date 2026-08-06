use gpui::prelude::FluentBuilder;
use gpui::*;
use raw_window_handle::HasWindowHandle;
use std::time::{Duration, Instant};

use crate::model::{BreakState, StretchlyConfig, StretchlyModel};
use crate::tips::current_tip;
use widget_core::AppConfig;

// ══════════════════════════════════════════════════════════════════════
// BreakOverlay — 每块显示器各一个独立全屏窗口，负责完整的休息 UI
// ══════════════════════════════════════════════════════════════════════

struct BreakOverlay {
    /// 是否已完成首次窗口样式设置
    styled: bool,
    /// 此覆盖窗口是否是主屏（主屏显示完整休息 UI，副屏只显示背景色）
    is_primary: bool,
}

impl Render for BreakOverlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 首次渲染：设置窗口置顶 + 移除可拖拽标题栏（不改变位置和大小）
        if !self.styled {
            self.styled = true;
            if let Ok(h) = window.window_handle() {
                if let raw_window_handle::RawWindowHandle::Win32(h) = h.as_raw() {
                    let hwnd = h.hwnd.get();
                    unsafe {
                        use windows_sys::Win32::UI::WindowsAndMessaging::*;
                        // 置顶，不改变位置和大小
                        SetWindowPos(
                            hwnd,
                            HWND_TOPMOST,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                        );
                        // 移除标题栏和边框样式，防止拖动
                        let style = GetWindowLongW(hwnd, GWL_STYLE);
                        SetWindowLongW(
                            hwnd,
                            GWL_STYLE,
                            (style & !(WS_CAPTION as i32) & !(WS_THICKFRAME as i32))
                                | WS_POPUP as i32,
                        );
                    }
                }
            }
        }
        // 监听休息结束信号：自毁
        let break_active = cx
            .try_global::<crate::StretchlyBreakActive>()
            .is_some_and(|g| g.0);
        if !break_active {
            window.remove_window();
        }

        // 副屏只显示背景色
        if !self.is_primary {
            return div().size_full().bg(rgba(0x02050eb0u32));
        }

        // ── 从全局快照读取渲染数据（无跨 Entity 借用风险）─────────────────────────────
        let snap = cx
            .try_global::<crate::StretchlyBreakSnapshot>()
            .cloned()
            .unwrap_or_default();

        let accent_color = if snap.is_mini {
            rgb(0x34d399u32)
        } else {
            rgb(0xa78bfau32)
        };
        let accent_bg = if snap.is_mini {
            rgba(0x34d39920u32)
        } else {
            rgba(0xa78bfa20u32)
        };

        div()
            .size_full()
            .bg(rgba(0x02050eb0u32))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .absolute()
                    .w(px(600.0))
                    .h(px(600.0))
                    .rounded(px(300.0))
                    .bg(rgba(0x0d1f3510u32))
                    .flex_shrink_0(),
            )
            .child(
                div()
                    .w(px(560.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(20.0))
                    .px(px(48.0))
                    .py(px(44.0))
                    .bg(rgba(0x0a1628ccu32))
                    .rounded(px(28.0))
                    .border_1()
                    .border_color(rgba(0xffffff18u32))
                    // 标题区
                    .child(
                        div().flex().flex_col().items_center().gap(px(6.0)).child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .px(px(14.0))
                                .py(px(5.0))
                                .bg(accent_bg)
                                .rounded(px(20.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(accent_color)
                                        .child(snap.break_label),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgba(0xc8d4f070u32))
                                        .child(format!("· {}", snap.break_duration_label)),
                                ),
                        ),
                    )
                    // 大号倒计时
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xf1f5f9u32))
                                    .child(snap.time_str.clone()),
                            )
                            .child(
                                div()
                                    .w(px(280.0))
                                    .h(px(5.0))
                                    .rounded(px(3.0))
                                    .bg(rgba(0xffffff10u32))
                                    .child(
                                        div()
                                            .h_full()
                                            .rounded(px(3.0))
                                            .bg(accent_color)
                                            .w(relative(snap.progress)),
                                    ),
                            ),
                    )
                    // 休息建议
                    .child(
                        div()
                            .w_full()
                            .px(px(16.0))
                            .py(px(14.0))
                            .bg(rgba(0xffffff07u32))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(rgba(0xffffff0au32))
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgba(0x94a3b860u32))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("休息建议"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgba(0xcbd5e1b0u32))
                                    .text_center()
                                    .child(snap.tip.clone()),
                            ),
                    )
                    .child(div().w_full().h(px(1.0)).bg(rgba(0xffffff0au32)))
                    // 按钮行
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(10.0))
                            // 推迟
                            .child(
                                div()
                                    .px(px(18.0))
                                    .py(px(9.0))
                                    .rounded(px(8.0))
                                    .bg(rgba(0xffffff0cu32))
                                    .hover(|s| s.bg(rgba(0xffffff18u32)))
                                    .cursor_pointer()
                                    .text_sm()
                                    .text_color(rgba(0xc8d4f0a0u32))
                                    .id("postpone-break-btn")
                                    .on_click(cx.listener(|_, _: &ClickEvent, _, cx| {
                                        cx.set_global(crate::StretchlyOverlayRequest(Some(
                                            crate::StretchlyOverlayAction::Postpone,
                                        )));
                                    }))
                                    .child(format!("推迟 {} 分钟", snap.postpone_mins)),
                            )
                            // 结束休息
                            .child(
                                div()
                                    .px(px(18.0))
                                    .py(px(9.0))
                                    .rounded(px(8.0))
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .id("skip-break-btn")
                                    .when(snap.skip_available, |d| {
                                        d.bg(accent_bg)
                                            .hover(|s| s.bg(rgba(0x34d39930u32)))
                                            .cursor_pointer()
                                            .text_color(accent_color)
                                            .on_click(cx.listener(|_, _: &ClickEvent, _, cx| {
                                                cx.set_global(crate::StretchlyOverlayRequest(
                                                    Some(crate::StretchlyOverlayAction::Skip),
                                                ));
                                            }))
                                    })
                                    .when(!snap.skip_available, |d| {
                                        d.bg(rgba(0xffffff08u32)).text_color(rgba(0xffffff35u32))
                                    })
                                    .child(snap.skip_label.clone()),
                            ),
                    ),
            )
    }
}

/// 枚举所有显示器的坐标，返回 (x, y, w, h, is_primary)
/// is_primary = 该显示器是否是 widget_hwnd 所在的显示器
fn get_all_monitor_rects(widget_hwnd: isize) -> Vec<(i32, i32, i32, i32, bool)> {
    unsafe {
        use windows_sys::Win32::Foundation::{BOOL, RECT};
        use windows_sys::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, MONITORINFO,
            MONITOR_DEFAULTTONEAREST,
        };

        let widget_mon = MonitorFromWindow(widget_hwnd, MONITOR_DEFAULTTONEAREST);

        struct State {
            widget_mon: isize,
            rects: Vec<(i32, i32, i32, i32, bool)>,
        }
        let mut state = State {
            widget_mon,
            rects: Vec::new(),
        };

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
            let is_primary = hmon == s.widget_mon;
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
                    .timer(Duration::from_secs(1))
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
                                // 隐藏小组件窗口（避免与全屏遮罩重叠）
                                if hwnd != 0 {
                                    unsafe {
                                        use windows_sys::Win32::UI::WindowsAndMessaging::{
                                            ShowWindow, SW_HIDE,
                                        };
                                        ShowWindow(hwnd, SW_HIDE);
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
                                                cx.observe_global::<crate::StretchlyBreakSnapshot>(
                                                    |_, cx| cx.notify(),
                                                )
                                                .detach();
                                                cx.observe_global::<crate::StretchlyBreakActive>(
                                                    |_, cx| cx.notify(),
                                                )
                                                .detach();
                                                BreakOverlay {
                                                    styled: false,
                                                    is_primary,
                                                }
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
                                            ShowWindow, SW_SHOWNA,
                                        };
                                        ShowWindow(hwnd, SW_SHOWNA);
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

impl Render for StretchlyWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit_mode = cx
            .try_global::<widget_core::UIState>()
            .is_some_and(|s| s.is_edit_mode);
        widget_core::update_window_edit_mode(window, is_edit_mode);

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

        // ── 编辑模式拖拽条 ────────────────────────────────────────────────────
        let drag_handle = if is_edit_mode && !is_on_break {
            Some(
                div()
                    .w_full()
                    .h(px(24.0))
                    .bg(rgb(0x00d992))
                    .flex()
                    .justify_center()
                    .items_center()
                    .flex_shrink_0()
                    .id("stretchly-drag")
                    .on_mouse_down(MouseButton::Left, |_, win, _| {
                        widget_core::start_window_drag(win);
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x050507))
                            .child(":: 拖拽移动 ::"),
                    ),
            )
        } else {
            None
        };

        // ══════════════════════════════════════════════════════════════════════
        // 休息中：全屏遮罩（P2: 视觉优化）
        // ══════════════════════════════════════════════════════════════════════
        if is_on_break {
            return div();
        }

        // ══════════════════════════════════════════════════════════════════════
        // 工作中：紧凑小组件
        // ══════════════════════════════════════════════════════════════════════

        let bg_color = if is_warning {
            rgba(0x1a0e04efu32)
        } else {
            rgba(0x0d111aefu32)
        };
        let border_color = if is_warning {
            rgba(0xf59e0b50u32)
        } else {
            rgba(0xffffff18u32)
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
            .size_full()
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .children(drag_handle)
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
                                    .when(is_warning, |d| {
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
                                    // 正常工作时：立即休息按钮
                                    .when(!is_warning && !is_paused, |d| {
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
