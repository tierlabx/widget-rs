use gpui::prelude::FluentBuilder;
use gpui::*;
use raw_window_handle::HasWindowHandle;
use std::time::{Duration, Instant};

use crate::model::{BreakState, StretchlyConfig, StretchlyModel};
use crate::tips::current_tip;
use widget_core::AppConfig;

struct OriginalBounds {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

// ══════════════════════════════════════════════════════════════════════
// 副屏遮罩：休息开始时在每块非主显示器上各创建一个同色覆盖层
// ══════════════════════════════════════════════════════════════════════

struct SecondaryOverlay {
    /// 是否已完成首次全屏定位
    positioned: bool,
}

impl Render for SecondaryOverlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 首次渲染：用 SetWindowPos 覆盖本显示器全屏
        if !self.positioned {
            self.positioned = true;
            if let Ok(h) = window.window_handle() {
                if let raw_window_handle::RawWindowHandle::Win32(h) = h.as_raw() {
                    let hwnd = h.hwnd.get();
                    secondary_enter_fullscreen(hwnd);
                }
            }
        }
        // 监听休息结束信号：自毁
        let break_active = cx
            .try_global::<crate::StretchlyBreakActive>()
            .is_some_and(|g| g.0);
        if !break_active {
            if let Ok(h) = window.window_handle() {
                if let raw_window_handle::RawWindowHandle::Win32(h) = h.as_raw() {
                    unsafe {
                        use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
                        PostMessageW(h.hwnd.get(), WM_CLOSE, 0, 0);
                    }
                }
            }
        }
        div().size_full().bg(rgba(0x02050eb0u32))
    }
}

/// 将副屏窗口全屏铺满其所在显示器（含 +8px 扩展消除缝隙）
fn secondary_enter_fullscreen(hwnd: isize) {
    unsafe {
        use windows_sys::Win32::Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_SHOWWINDOW,
        };
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(hmon, &mut info as *mut _);
        let m = info.rcMonitor;
        let extra = 8i32;
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            m.left - extra,
            m.top - extra,
            (m.right - m.left) + extra * 2,
            (m.bottom - m.top) + extra * 2,
            SWP_FRAMECHANGED | SWP_SHOWWINDOW,
        );
    }
}

/// 枚举除 widget_hwnd 所在显示器以外的所有显示器坐标
fn get_secondary_monitor_rects(widget_hwnd: isize) -> Vec<(i32, i32, i32, i32)> {
    unsafe {
        use windows_sys::Win32::Foundation::{BOOL, RECT};
        use windows_sys::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, MONITORINFO,
            MONITOR_DEFAULTTONEAREST,
        };

        let widget_mon = MonitorFromWindow(widget_hwnd, MONITOR_DEFAULTTONEAREST);

        struct State {
            widget_mon: isize,
            rects: Vec<(i32, i32, i32, i32)>,
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
            if hmon != s.widget_mon {
                let mut info: MONITORINFO = std::mem::zeroed();
                info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                GetMonitorInfoW(hmon, &mut info as *mut _);
                let r = info.rcMonitor;
                s.rects
                    .push((r.left, r.top, r.right - r.left, r.bottom - r.top));
            }
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
    original_bounds: Option<OriginalBounds>,
    /// 当前休息阶段的开始时刻（用于计算跳过延迟）
    break_started_at: Option<Instant>,
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
            original_bounds: None,
            break_started_at: None,
            _timer,
        }
    }

    // ── Win32 窗口操作 ────────────────────────────────────────────────────────

    /// 全屏覆盖小组件所在显示器（含 +8px 强制扩展，消除 DWM / GPUI 边缘缝隙）
    fn enter_fullscreen(hwnd: isize) {
        unsafe {
            use windows_sys::Win32::Graphics::Gdi::{
                GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                SetWindowPos, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_SHOWWINDOW,
            };
            let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(hmon, &mut info as *mut _);
            let m = info.rcMonitor;
            // 扩展 8px 以覆盖 DWM 阴影 / GPUI 内边距导致的任何边缘空白
            let extra = 8i32;
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                m.left - extra,
                m.top - extra,
                (m.right - m.left) + extra * 2,
                (m.bottom - m.top) + extra * 2,
                SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
        }
    }

    fn exit_fullscreen(hwnd: isize, b: &OriginalBounds) {
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                SetWindowPos, HWND_NOTOPMOST, SWP_FRAMECHANGED, SWP_SHOWWINDOW,
            };
            SetWindowPos(
                hwnd,
                HWND_NOTOPMOST,
                b.x,
                b.y,
                b.w,
                b.h,
                SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
        }
    }

    fn get_window_bounds(hwnd: isize) -> Option<OriginalBounds> {
        unsafe {
            use windows_sys::Win32::Foundation::RECT;
            use windows_sys::Win32::Graphics::Dwm::{
                DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS,
            };
            let mut rect: RECT = std::mem::zeroed();
            let hr = DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS as u32,
                &mut rect as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            );
            if hr == 0 {
                return Some(OriginalBounds {
                    x: rect.left,
                    y: rect.top,
                    w: rect.right - rect.left,
                    h: rect.bottom - rect.top,
                });
            }
            use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect;
            if GetWindowRect(hwnd, &mut rect) != 0 {
                return Some(OriginalBounds {
                    x: rect.left,
                    y: rect.top,
                    w: rect.right - rect.left,
                    h: rect.bottom - rect.top,
                });
            }
            None
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

        // ── 获取 HWND ────────────────────────────────────────────────────────
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

        // ── 状态切换：全屏 / 还原 + 副屏遮罩 ───────────────────────────────────
        let is_on_break = self.model.is_on_break();
        if self.was_working != !is_on_break {
            self.was_working = !is_on_break;
            if is_on_break {
                if hwnd != 0 {
                    if self.original_bounds.is_none() {
                        self.original_bounds = Self::get_window_bounds(hwnd);
                    }
                    Self::enter_fullscreen(hwnd);
                    // 多显示器：为每块副屏开启独立遮罩窗口
                    cx.set_global(crate::StretchlyBreakActive(true));
                    for (x, y, w, h) in get_secondary_monitor_rects(hwnd) {
                        let _ = cx.open_window(
                            WindowOptions {
                                titlebar: None,
                                window_background: WindowBackgroundAppearance::Transparent,
                                kind: WindowKind::PopUp,
                                is_resizable: false,
                                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                                    Point::new(px(x as f32), px(y as f32)),
                                    size(px(w as f32), px(h as f32)),
                                ))),
                                ..Default::default()
                            },
                            |window, cx| {
                                let view = cx.new(|_| SecondaryOverlay { positioned: false });
                                cx.new(|cx| gpui_component::Root::new(view, window, cx))
                            },
                        );
                    }
                }
                self.break_started_at = Some(Instant::now());
            } else {
                if hwnd != 0 {
                    if let Some(ref b) = self.original_bounds {
                        Self::exit_fullscreen(hwnd, b);
                    }
                }
                // 通知副屏遮罩窗口关闭自身
                cx.set_global(crate::StretchlyBreakActive(false));
                self.break_started_at = None;
            }
        }

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

        let skip_delay = self.model.config.skip_delay_seconds;
        let break_elapsed_secs = self
            .break_started_at
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);
        let skip_available = break_elapsed_secs >= skip_delay;
        let skip_countdown = skip_delay.saturating_sub(break_elapsed_secs);
        let postpone_mins = self.model.config.postpone_minutes;

        let rem_mins = remaining.as_secs() / 60;
        let rem_secs = remaining.as_secs() % 60;
        let time_str = if is_on_break {
            format!("{:02}:{:02}", rem_mins, rem_secs)
        } else if rem_mins >= 1 {
            format!("约 {} 分钟", rem_mins + if rem_secs >= 30 { 1 } else { 0 })
        } else {
            format!("{} 秒", rem_secs)
        };

        let tip = current_tip();

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
            let (break_label, accent_color, accent_bg) = match self.model.state {
                BreakState::MiniBreak => ("微休", rgb(0x34d399u32), rgba(0x34d39920u32)),
                BreakState::LongBreak => ("长休", rgb(0xa78bfau32), rgba(0xa78bfa20u32)),
                BreakState::Working => ("", rgb(0x34d399u32), rgba(0x34d39920u32)),
            };
            let break_duration_label = match self.model.state {
                BreakState::MiniBreak => {
                    format!("{} 秒", self.model.config.mini_break_duration)
                }
                BreakState::LongBreak => {
                    format!("{} 分钟", self.model.config.long_break_duration / 60)
                }
                BreakState::Working => String::new(),
            };
            let cycle_label = if self.model.state == BreakState::MiniBreak {
                format!("第 {} / {} 次微休", mini_taken, mini_total)
            } else {
                "完整长休".to_string()
            };
            let skip_label = if skip_available {
                "结束休息".to_string()
            } else {
                format!("结束休息 ({}s)", skip_countdown)
            };

            // P2: 背景使用深色 + 中央辉光层，营造深邃感
            return div()
                .size_full()
                .bg(rgba(0x02050eb0u32)) // 半透明深蓝黑（约 69% 不透明）
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                // P2: 中央柔光晕圈（大号半透明圆，营造焦点感）
                .child(
                    div()
                        .absolute()
                        .w(px(600.0))
                        .h(px(600.0))
                        .rounded(px(300.0))
                        .bg(rgba(0x0d1f3510u32)) // 极浅的冷蓝光晕
                        .flex_shrink_0(),
                )
                .child(
                    // ── 中央内容卡片 ─────────────────────────────────────────
                    div()
                        .w(px(560.0))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(20.0))
                        .px(px(48.0))
                        .py(px(44.0))
                        .bg(rgba(0xffffff09u32))
                        .rounded(px(28.0))
                        // 卡片边框（极细，增加精致感）
                        .border_1()
                        .border_color(rgba(0xffffff0du32))
                        // ── 标题区 ──────────────────────────────────────────
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(6.0))
                                // 类型 + 时长徽章
                                .child(
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
                                                .child(break_label),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgba(0xc8d4f070u32))
                                                .child(format!("· {}", break_duration_label)),
                                        ),
                                )
                                // 周期信息
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgba(0x94a3b870u32))
                                        .child(cycle_label),
                                ),
                        )
                        // ── 大号倒计时（P2: 更大字号）────────────────────────
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
                                        .child(time_str),
                                )
                                // 进度条（P2: 更粗，配色与类型对应）
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
                                                .w(relative(progress)),
                                        ),
                                ),
                        )
                        // ── 休息建议（P2: 更精致卡片）────────────────────────
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
                                        .child(tip),
                                ),
                        )
                        // ── 分隔线 ────────────────────────────────────────────
                        .child(div().w_full().h(px(1.0)).bg(rgba(0xffffff0au32)))
                        // ── 按钮行 ────────────────────────────────────────────
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap(px(10.0))
                                // 推迟按钮（语义正确：结束休息 + 延后下次）
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
                                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                            this.model.skip_and_postpone();
                                            cx.notify();
                                        }))
                                        .child(format!("推迟 {} 分钟", postpone_mins)),
                                )
                                // 结束休息按钮（有延迟保护）
                                .child(
                                    div()
                                        .px(px(18.0))
                                        .py(px(9.0))
                                        .rounded(px(8.0))
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .id("skip-break-btn")
                                        .when(skip_available, |d| {
                                            d.bg(accent_bg)
                                                .hover(|s| s.bg(rgba(0x34d39930u32)))
                                                .cursor_pointer()
                                                .text_color(accent_color)
                                                .on_click(cx.listener(
                                                    |this, _: &ClickEvent, _, cx| {
                                                        this.model.skip();
                                                        cx.notify();
                                                    },
                                                ))
                                        })
                                        .when(!skip_available, |d| {
                                            d.bg(rgba(0xffffff08u32))
                                                .text_color(rgba(0xffffff35u32))
                                        })
                                        .child(skip_label),
                                ),
                        ),
                );
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
