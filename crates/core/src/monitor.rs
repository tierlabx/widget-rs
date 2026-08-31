use gpui::App;

use crate::AppConfig;

/// 显示器物理矩形区域 (左, 上, 右, 下)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    #[inline]
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    #[inline]
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    #[inline]
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    /// 计算与另一个矩形的重叠相交面积
    pub fn overlap_area(&self, other: &Rect) -> i64 {
        let inter_left = self.left.max(other.left);
        let inter_top = self.top.max(other.top);
        let inter_right = self.right.min(other.right);
        let inter_bottom = self.bottom.min(other.bottom);

        let w = (inter_right - inter_left).max(0) as i64;
        let h = (inter_bottom - inter_top).max(0) as i64;
        w * h
    }
}

/// 完整显示器信息（物理像素坐标及 DPI）
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// 完整显示区域（含任务栏）
    pub rc_monitor: Rect,
    /// 可用工作区域（排除任务栏和固定停靠栏）
    pub rc_work: Rect,
    /// 水平 DPI
    pub dpi_x: u32,
    /// 垂直 DPI
    pub dpi_y: u32,
    /// 缩放系数 (DPI / 96.0)
    pub scale_factor: f32,
    /// 是否为主显示器
    pub is_primary: bool,
}

/// 枚举所有活跃显示器，返回完整的物理像素坐标与 DPI 信息
pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::{BOOL, RECT};
        use windows_sys::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MONITORINFO,
        };

        struct State {
            monitors: Vec<MonitorInfo>,
        }

        unsafe extern "system" fn callback(
            hmon: isize,
            _hdc: isize,
            _lp_rect: *mut RECT,
            lparam: isize,
        ) -> BOOL {
            let s = &mut *(lparam as *mut State);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(hmon, &mut info as *mut _);

            let rc_mon = Rect {
                left: info.rcMonitor.left,
                top: info.rcMonitor.top,
                right: info.rcMonitor.right,
                bottom: info.rcMonitor.bottom,
            };
            let rc_work = Rect {
                left: info.rcWork.left,
                top: info.rcWork.top,
                right: info.rcWork.right,
                bottom: info.rcWork.bottom,
            };

            let mut dpi_x: u32 = 96;
            let mut dpi_y: u32 = 96;
            let _ = windows_sys::Win32::UI::HiDpi::GetDpiForMonitor(
                hmon, 0, // MDT_EFFECTIVE_DPI
                &mut dpi_x, &mut dpi_y,
            );

            let is_primary = (info.dwFlags & 1) != 0; // MONITORINFOF_PRIMARY = 1
            let scale_factor = (dpi_x as f32 / 96.0).max(0.5);

            s.monitors.push(MonitorInfo {
                rc_monitor: rc_mon,
                rc_work,
                dpi_x,
                dpi_y,
                scale_factor,
                is_primary,
            });
            1
        }

        let mut state = State {
            monitors: Vec::new(),
        };

        unsafe {
            EnumDisplayMonitors(
                0,
                std::ptr::null(),
                Some(callback),
                &mut state as *mut State as isize,
            );
        }

        state.monitors
    }

    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
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

    // 1. 如果窗口有尺寸，按重叠相交面积最大优先匹配
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

    // 2. 如果无重叠（例如点或刚移出），测试中心点是否在显示器内
    let cx = px + pw / 2;
    let cy = py + ph / 2;
    if let Some(m) = monitors
        .iter()
        .find(|m| m.rc_monitor.contains_point(cx, cy))
    {
        return Some(m);
    }

    // 3. 测试左上角原点是否在显示器内
    if let Some(m) = monitors
        .iter()
        .find(|m| m.rc_monitor.contains_point(px, py))
    {
        return Some(m);
    }

    None
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

/// 恢复插件窗口的近似逻辑坐标（用于 GPUI 初始窗口创建）
///
/// 进行严谨的显示器几何校验与跨屏逻辑推导，返回的逻辑坐标用于让 GPUI 在正确的显示器和位置初始化。
pub fn resolve_plugin_bounds(
    cx: &App,
    plugin_id: &str,
    default: (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    let plugin_cfg = cx
        .try_global::<AppConfig>()
        .and_then(|cfg| cfg.plugins.get(plugin_id).cloned());

    let Some(p) = plugin_cfg else {
        return default;
    };

    if p.width <= 0.0 || p.height <= 0.0 {
        return default;
    }

    let monitors = enumerate_monitors();
    if monitors.is_empty() {
        return default;
    }

    for (i, m) in monitors.iter().enumerate() {
        println!(
            "[resolve_plugin_bounds] 显示器{}: 工作区 ({},{})~({},{}) DPI={} 缩放={}% 主屏={}",
            i,
            m.rc_work.left,
            m.rc_work.top,
            m.rc_work.right,
            m.rc_work.bottom,
            m.dpi_x,
            (m.scale_factor * 100.0) as u32,
            m.is_primary
        );
    }

    // 对于固定尺寸小组件（如 stretchly 药丸小部件），逻辑尺寸锁定为设计紧凑尺寸，只精准解析屏幕位置
    if plugin_id == "stretchly_widget" {
        let (fixed_w, fixed_h) = (default.2, default.3);
        if p.phys_w > 0 && p.phys_h > 0 {
            if let Some(m) = find_best_monitor(&monitors, p.phys_x, p.phys_y, p.phys_w, p.phys_h) {
                let scale = m.scale_factor;
                let log_x = p.phys_x as f32 / scale;
                let log_y = p.phys_y as f32 / scale;
                return (log_x, log_y, fixed_w, fixed_h);
            }
        }
        return (p.x, p.y, fixed_w, fixed_h);
    }

    // 优先使用物理坐标进行多屏定位匹配
    if p.phys_w > 0 && p.phys_h > 0 {
        if let Some(m) = find_best_monitor(&monitors, p.phys_x, p.phys_y, p.phys_w, p.phys_h) {
            let scale = m.scale_factor;
            let log_x = p.phys_x as f32 / scale;
            let log_y = p.phys_y as f32 / scale;
            let log_w = p.phys_w as f32 / scale;
            let log_h = p.phys_h as f32 / scale;
            return (log_x, log_y, log_w, log_h);
        }
    }

    // 回退尝试使用逻辑坐标
    let s = if p.scale > 0.0 { p.scale } else { 1.0 };
    let approx_px = (p.x * s).round() as i32;
    let approx_py = (p.y * s).round() as i32;
    let approx_pw = (p.width * s).round() as i32;
    let approx_ph = (p.height * s).round() as i32;

    if find_best_monitor(&monitors, approx_px, approx_py, approx_pw, approx_ph).is_some() {
        (p.x, p.y, p.width, p.height)
    } else {
        println!(
            "[resolve_plugin_bounds] 插件 {} 坐标不在任何活跃显示器上，回退默认位置",
            plugin_id
        );
        default
    }
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

    // 直接使用保存的绝对物理坐标（在目标显示器内经 clamp 保证安全），杜绝多次启动反复缩放放大的恶性累积
    let clamped = clamp_to_work_area(monitor, p.phys_x, p.phys_y, phys_w, phys_h);
    Some(clamped)
}
