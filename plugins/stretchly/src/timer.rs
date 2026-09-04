use gpui::*;
use std::time::{Duration, Instant};

use crate::model::{BreakState, StretchlyConfig};
use crate::overlay::BreakOverlay;
use crate::tips::current_tip;
use crate::view::StretchlyWidget;
use widget_core::AppConfig;

/// 枚举所有显示器的物理坐标，返回 (x, y, w, h, is_primary)
pub fn get_all_monitor_rects() -> Vec<(i32, i32, i32, i32, bool)> {
    unsafe {
        use windows_sys::Win32::Foundation::{BOOL, RECT};
        use windows_sys::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MONITORINFO,
        };

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
            let is_primary = (info.dwFlags & 1) != 0;
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

/// 启动 Stretchly 智能自适应心跳后台任务
///
/// - 休息中 / 预警中 / 最后 10 秒：100ms 刷新率，保证动画与进度条丝滑
/// - 正常专注工作期：自适应降频至 500ms~1000ms，大幅降低空闲 CPU 与内存消耗
pub fn spawn_stretchly_timer(
    this: WeakEntity<StretchlyWidget>,
    cx: &mut Context<StretchlyWidget>,
) -> gpui::Task<()> {
    let app_cx: &mut App = cx;
    app_cx.spawn(async move |async_cx| {
        let mut last_remaining_secs = u64::MAX;
        let mut last_progress_bucket = 0u32;
        let mut last_state = BreakState::Working;
        let mut last_paused = false;

        loop {
            // 默认心跳间隔：根据当前状态动态自适应计算
            let (is_high_freq, interval_ms) = async_cx.update(|cx| {
                this.upgrade()
                    .map(|entity| {
                        let widget = entity.read(cx);
                        let on_break = widget.model().is_on_break();
                        let warning = widget.model().is_warning();
                        let rem_secs = widget.model().time_remaining().as_secs();
                        let high_freq = on_break || warning || rem_secs <= 10;
                        let ms = if high_freq { 100 } else { 500 };
                        (high_freq, ms)
                    })
                    .unwrap_or((false, 500))
            });

            let _ = is_high_freq;
            async_cx
                .background_executor()
                .timer(Duration::from_millis(interval_ms))
                .await;

            let is_active = async_cx.update(|cx| {
                this.update(cx, |this, cx| {
                    // 1. 处理 BreakOverlay 按钮回调请求
                    if let Some(req) = cx
                        .try_global::<crate::StretchlyOverlayRequest>()
                        .and_then(|r| r.0.clone())
                    {
                        cx.set_global(crate::StretchlyOverlayRequest(None));
                        match req {
                            crate::StretchlyOverlayAction::Skip => this.model_mut().skip(),
                            crate::StretchlyOverlayAction::Postpone => {
                                this.model_mut().skip_and_postpone()
                            }
                        }
                    }

                    // 2. 热更新配置
                    if let Some(cfg) = cx.try_global::<AppConfig>() {
                        if let Some(new_cfg) =
                            cfg.get_plugin_data::<StretchlyConfig>("stretchly_widget")
                        {
                            let apply_now = cx
                                .try_global::<crate::StretchlyApplyNow>()
                                .is_some_and(|s| s.0);
                            if apply_now {
                                this.model_mut().apply_config_now(new_cfg);
                                cx.set_global(crate::StretchlyApplyNow(false));
                            } else {
                                this.model_mut().queue_config_update(new_cfg);
                            }
                        }
                    }

                    this.model_mut().tick();
                    cx.set_global(crate::StretchlyLiveStats(this.model().stats.clone()));

                    // 3. 发布休息快照
                    if this.model().is_on_break() {
                        let remaining = this.model().time_remaining();
                        let rem_mins = remaining.as_secs() / 60;
                        let rem_secs = remaining.as_secs() % 60;
                        let time_str = format!("{:02}:{:02}", rem_mins, rem_secs);
                        let skip_delay = this.model().config.skip_delay_seconds;
                        let break_elapsed_secs = this
                            .break_started_at()
                            .map(|t| t.elapsed().as_secs())
                            .unwrap_or(0);
                        let skip_available = break_elapsed_secs >= skip_delay;
                        let skip_countdown = skip_delay.saturating_sub(break_elapsed_secs);
                        let (is_mini, break_label, break_duration_label) = match this.model().state
                        {
                            BreakState::MiniBreak => (
                                true,
                                "微休",
                                format!("{} 秒", this.model().config.mini_break_duration),
                            ),
                            BreakState::LongBreak => (
                                false,
                                "长休",
                                format!("{} 分钟", this.model().config.long_break_duration / 60),
                            ),
                            BreakState::Working => (true, "", String::new()),
                        };
                        cx.set_global(crate::StretchlyBreakSnapshot {
                            state: this.model().state,
                            time_str,
                            progress: this.model().progress(),
                            break_label,
                            break_duration_label,
                            is_mini,
                            skip_available,
                            skip_label: if skip_available {
                                "结束休息".to_string()
                            } else {
                                format!("结束休息 ({}s)", skip_countdown)
                            },
                            postpone_mins: this.model().config.postpone_minutes,
                            tip: current_tip().to_string(),
                            allow_skip: this.model().config.allow_skip,
                            allow_postpone: this.model().config.allow_postpone,
                        });
                    }

                    // 4. 状态切换与遮罩管理
                    let is_on_break = this.model().is_on_break();
                    if this.was_working() != !is_on_break {
                        this.set_was_working(!is_on_break);
                        let hwnd = this.cached_hwnd();
                        if is_on_break {
                            cx.set_global(crate::StretchlyBreakActive(true));
                            this.set_break_started_at(Some(Instant::now()));
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
                            for (x, y, w, h, is_primary) in get_all_monitor_rects() {
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
                                    move |_window, cx| {
                                        cx.new(|cx| {
                                            let sub1 = cx
                                                .observe_global::<crate::StretchlyBreakSnapshot>(
                                                    |_, cx| cx.notify(),
                                                );
                                            let sub2 = cx
                                                .observe_global::<crate::StretchlyBreakActive>(
                                                    |_, cx| cx.notify(),
                                                );
                                            BreakOverlay::new(
                                                is_primary,
                                                (x, y, w, h),
                                                vec![sub1, sub2],
                                            )
                                        })
                                    },
                                );
                            }
                        } else {
                            cx.set_global(crate::StretchlyBreakActive(false));
                            this.set_break_started_at(None);
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

                    // 5. 智能重绘触发：仅在数据实质变化时通知 GPUI
                    let curr_rem_secs = this.model().time_remaining().as_secs();
                    let curr_progress_bucket = (this.model().progress() * 200.0) as u32;
                    let curr_state = this.model().state;
                    let curr_paused = this.model().is_paused;

                    if curr_rem_secs != last_remaining_secs
                        || curr_progress_bucket != last_progress_bucket
                        || curr_state != last_state
                        || curr_paused != last_paused
                    {
                        last_remaining_secs = curr_rem_secs;
                        last_progress_bucket = curr_progress_bucket;
                        last_state = curr_state;
                        last_paused = curr_paused;
                        cx.notify();
                    }
                })
                .is_ok()
            });

            if !is_active {
                break;
            }
        }
    })
}
