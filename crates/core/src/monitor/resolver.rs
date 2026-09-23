use gpui::{App, DisplayId};

use super::types::{MonitorInfo, Rect, ResolvedPluginBounds};
use super::win32::enumerate_monitors;
use crate::AppConfig;

/// 获取主显示器信息（用于 GPUI 全局逻辑坐标基准查询）
pub fn find_primary_monitor(monitors: &[MonitorInfo]) -> Option<&MonitorInfo> {
    monitors
        .iter()
        .find(|m| m.is_primary)
        .or_else(|| monitors.first())
}

/// 根据物理矩形或中心点匹配最佳目标显示器（重叠面积最大优先）
pub fn find_best_monitor(
    monitors: &[MonitorInfo],
    px: i32,
    py: i32,
    pw: i32,
    ph: i32,
) -> Option<&MonitorInfo> {
    if monitors.is_empty() {
        return None;
    }

    let win_rect = Rect {
        left: px,
        top: py,
        right: px + pw.max(1),
        bottom: py + ph.max(1),
    };

    let mut best_m: Option<(&MonitorInfo, i64)> = None;
    for m in monitors {
        let area = m.rc_monitor.overlap_area(&win_rect);
        if area > 0 {
            if let Some((_, best_area)) = best_m {
                if area > best_area {
                    best_m = Some((m, area));
                }
            } else {
                best_m = Some((m, area));
            }
        }
    }
    if let Some((m, _)) = best_m {
        return Some(m);
    }

    let cx = px + pw / 2;
    let cy = py + ph / 2;
    if let Some(m) = monitors
        .iter()
        .find(|m| m.rc_monitor.contains_point(cx, cy))
    {
        return Some(m);
    }

    if let Some(m) = monitors
        .iter()
        .find(|m| m.rc_monitor.contains_point(px, py))
    {
        return Some(m);
    }

    // 最终回退：主显示器或首个显示器（防止离屏丢失）
    monitors
        .iter()
        .find(|m| m.is_primary)
        .or_else(|| monitors.first())
}

/// 将窗口物理坐标安全约束在目标显示器的工作区内（防止超屏隐藏或无法操作）
pub fn clamp_to_work_area(
    m: &MonitorInfo,
    mut px: i32,
    mut py: i32,
    mut pw: i32,
    mut ph: i32,
) -> (i32, i32, i32, i32) {
    let max_w = m.rc_work.width();
    let max_h = m.rc_work.height();

    if pw > max_w {
        pw = max_w;
    }
    if ph > max_h {
        ph = max_h;
    }

    let min_x = m.rc_work.left;
    let max_x = (m.rc_work.right - pw).max(min_x);
    if px < min_x {
        px = min_x;
    } else if px > max_x {
        px = max_x;
    }

    let min_y = m.rc_work.top;
    let max_y = (m.rc_work.bottom - ph).max(min_y);
    if py < min_y {
        py = min_y;
    } else if py > max_y {
        py = max_y;
    }

    (px, py, pw, ph)
}

/// 在 GPUI 活跃显示器中匹配包含给定逻辑坐标矩形的 DisplayId
pub fn match_display_for_logical_bounds(
    cx: &App,
    log_x: f32,
    log_y: f32,
    log_w: f32,
    log_h: f32,
) -> Option<DisplayId> {
    let displays = cx.displays();
    if displays.is_empty() {
        return None;
    }

    let center_x = log_x + log_w / 2.0;
    let center_y = log_y + log_h / 2.0;

    // 1. 优先匹配中心点落入的显示器
    for d in &displays {
        let b = d.bounds();
        let left: f32 = b.origin.x.into();
        let top: f32 = b.origin.y.into();
        let right: f32 = left + f32::from(b.size.width);
        let bottom: f32 = top + f32::from(b.size.height);

        if center_x >= left && center_x < right && center_y >= top && center_y < bottom {
            return Some(d.id());
        }
    }

    // 2. 次选匹配有重叠相交面积的显示器
    let mut best_overlap: Option<(DisplayId, f32)> = None;
    for d in &displays {
        let b = d.bounds();
        let left: f32 = b.origin.x.into();
        let top: f32 = b.origin.y.into();
        let right: f32 = left + f32::from(b.size.width);
        let bottom: f32 = top + f32::from(b.size.height);

        let inter_w = (log_x + log_w).min(right) - log_x.max(left);
        let inter_h = (log_y + log_h).min(bottom) - log_y.max(top);
        if inter_w > 0.0 && inter_h > 0.0 {
            let area = inter_w * inter_h;
            if let Some((_, best_area)) = best_overlap {
                if area > best_area {
                    best_overlap = Some((d.id(), area));
                }
            } else {
                best_overlap = Some((d.id(), area));
            }
        }
    }

    if let Some((id, _)) = best_overlap {
        return Some(id);
    }

    // 3. 回退主屏或第一个显示器
    cx.primary_display()
        .map(|d| d.id())
        .or_else(|| displays.first().map(|d| d.id()))
}

/// 解析插件窗口在 GPUI 中的初始逻辑位置与对应的目标 DisplayId
pub fn resolve_plugin_window_bounds(
    cx: &App,
    plugin_id: &str,
    default: (f32, f32, f32, f32),
) -> ResolvedPluginBounds {
    let primary_id = cx.primary_display().map(|d| d.id());
    let default_res = ResolvedPluginBounds {
        x: default.0,
        y: default.1,
        width: default.2,
        height: default.3,
        display_id: primary_id,
    };

    let plugin_cfg = cx
        .try_global::<AppConfig>()
        .and_then(|cfg| cfg.plugins.get(plugin_id).cloned());

    let Some(p) = plugin_cfg else {
        return default_res;
    };

    if p.width <= 0.0 || p.height <= 0.0 {
        return default_res;
    }

    let monitors = enumerate_monitors();
    if monitors.is_empty() {
        return default_res;
    }

    // 固定尺寸小部件（如 stretchly）尺寸锁定
    if plugin_id == "stretchly_widget" {
        let (fixed_w, fixed_h) = (default.2, default.3);
        if p.phys_w > 0 && p.phys_h > 0 {
            if let Some(m) = find_best_monitor(&monitors, p.phys_x, p.phys_y, p.phys_w, p.phys_h) {
                let scale = m.scale_factor;
                let log_x = p.phys_x as f32 / scale;
                let log_y = p.phys_y as f32 / scale;
                let display_id =
                    match_display_for_logical_bounds(cx, log_x, log_y, fixed_w, fixed_h);
                return ResolvedPluginBounds {
                    x: log_x,
                    y: log_y,
                    width: fixed_w,
                    height: fixed_h,
                    display_id,
                };
            }
        }
        let display_id = match_display_for_logical_bounds(cx, p.x, p.y, fixed_w, fixed_h);
        return ResolvedPluginBounds {
            x: p.x,
            y: p.y,
            width: fixed_w,
            height: fixed_h,
            display_id,
        };
    }

    // 优先使用物理像素坐标匹配目标显示器与计算逻辑坐标
    if p.phys_w > 0 && p.phys_h > 0 {
        if let Some(m) = find_best_monitor(&monitors, p.phys_x, p.phys_y, p.phys_w, p.phys_h) {
            let scale = m.scale_factor;
            let log_x = p.phys_x as f32 / scale;
            let log_y = p.phys_y as f32 / scale;
            let log_w = p.phys_w as f32 / scale;
            let log_h = p.phys_h as f32 / scale;
            let display_id = match_display_for_logical_bounds(cx, log_x, log_y, log_w, log_h);
            return ResolvedPluginBounds {
                x: log_x,
                y: log_y,
                width: log_w,
                height: log_h,
                display_id,
            };
        }
    }

    // 回退尝试使用逻辑坐标
    let s = if p.scale > 0.0 { p.scale } else { 1.0 };
    let approx_px = (p.x * s).round() as i32;
    let approx_py = (p.y * s).round() as i32;
    let approx_pw = (p.width * s).round() as i32;
    let approx_ph = (p.height * s).round() as i32;

    if find_best_monitor(&monitors, approx_px, approx_py, approx_pw, approx_ph).is_some() {
        let display_id = match_display_for_logical_bounds(cx, p.x, p.y, p.width, p.height);
        ResolvedPluginBounds {
            x: p.x,
            y: p.y,
            width: p.width,
            height: p.height,
            display_id,
        }
    } else {
        println!(
            "[resolve_plugin_bounds] 插件 {} 坐标不在任何活跃显示器上，安全回退主屏默认位置",
            plugin_id
        );
        default_res
    }
}

/// 兼容旧版调用，返回 (x, y, width, height) 逻辑边界
pub fn resolve_plugin_bounds(
    cx: &App,
    plugin_id: &str,
    default: (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    let res = resolve_plugin_window_bounds(cx, plugin_id, default);
    (res.x, res.y, res.width, res.height)
}

/// 获取插件已保存的物理像素坐标（经过显示器边界校验与安全工作区 Clamping）
pub fn get_saved_physical_bounds(cx: &App, plugin_id: &str) -> Option<(i32, i32, i32, i32)> {
    let p = cx
        .try_global::<AppConfig>()
        .and_then(|cfg| cfg.plugins.get(plugin_id).cloned())?;

    if p.phys_w <= 0 || p.phys_h <= 0 {
        return None;
    }

    let monitors = enumerate_monitors();
    let monitor = find_best_monitor(&monitors, p.phys_x, p.phys_y, p.phys_w, p.phys_h)?;

    let (phys_w, phys_h) = if plugin_id == "stretchly_widget" {
        let pw = (280.0 * monitor.scale_factor).round() as i32;
        let ph = (78.0 * monitor.scale_factor).round() as i32;
        (pw, ph)
    } else {
        (p.phys_w, p.phys_h)
    };

    let clamped = clamp_to_work_area(monitor, p.phys_x, p.phys_y, phys_w, phys_h);
    Some(clamped)
}
