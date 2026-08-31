use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, IsWindowVisible, WINDOWPOS, WMSZ_BOTTOM, WMSZ_BOTTOMLEFT, WMSZ_BOTTOMRIGHT,
    WMSZ_LEFT, WMSZ_RIGHT, WMSZ_TOP, WMSZ_TOPLEFT, WMSZ_TOPRIGHT,
};

use super::windows::WND_PROCS;

/// 矩形区域辅助结构
#[derive(Debug, Clone, Copy)]
pub struct SnapRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl SnapRect {
    #[inline]
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    #[inline]
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// 计算小组件窗口拖拽移动时的智能边缘吸附与网格化高度/水平对齐
///
/// 具备 DPI 动态感知、视觉精确边界对齐、相同高度自动吸附、网格间隙对齐等特性，
/// 消除隐形缩放边框缝隙，支持屏幕边缘 0 缝隙吸附、顶对齐、底对齐、垂直中心对齐与相贴磁吸。
pub unsafe fn apply_window_snapping(hwnd: isize, pos: &mut WINDOWPOS) {
    let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    let mut info: MONITORINFO = std::mem::zeroed();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    if GetMonitorInfoW(hmonitor, &mut info) == 0 {
        return;
    }

    let mut dpi_x: u32 = 96;
    let mut dpi_y: u32 = 96;
    let _ = windows_sys::Win32::UI::HiDpi::GetDpiForMonitor(
        hmonitor, 0, // MDT_EFFECTIVE_DPI
        &mut dpi_x, &mut dpi_y,
    );
    let scale = (dpi_x as f32 / 96.0).max(0.5);

    // DPI 动态自适应吸附阈值（100% 下 14px，150% 下 21px）
    let snap_threshold = ((14.0 * scale).round() as i32).clamp(10, 24);
    let proximity_max = snap_threshold * 4;
    // 标准网格间距（如 8px 网格）
    let grid_gap = ((8.0 * scale).round() as i32).max(4);

    let work = SnapRect {
        left: info.rcWork.left,
        top: info.rcWork.top,
        right: info.rcWork.right,
        bottom: info.rcWork.bottom,
    };

    let curr_w = pos.cx;
    let curr_h = pos.cy;
    let curr_x = pos.x;
    let curr_y = pos.y;

    // 收集所有其他可见小组件的真实视觉物理矩形（无隐形边框）
    let mut other_rects = Vec::new();
    if let Some(procs) = WND_PROCS.get() {
        if let Ok(guard) = procs.lock() {
            collect_visible_windows(hwnd, &guard, &mut other_rects);
        }
    }

    // ── X 轴候选求解（水平吸附与网格对齐）─────────────────────────
    let mut best_x: Option<(i32, i32)> = None; // (target_x, abs_delta)

    let mut consider_x = |target_x: i32| {
        let delta = (curr_x - target_x).abs();
        if delta < snap_threshold {
            if let Some((_, best_delta)) = best_x {
                if delta < best_delta {
                    best_x = Some((target_x, delta));
                }
            } else {
                best_x = Some((target_x, delta));
            }
        }
    };

    // 屏幕左边缘与右边缘紧贴（0 缝隙）
    consider_x(work.left);
    consider_x(work.right - curr_w);

    // 组件间水平对齐与相贴
    for other in &other_rects {
        let y_overlap = (curr_y + curr_h).min(other.bottom) - curr_y.max(other.top);
        let y_gap = if y_overlap < 0 { -y_overlap } else { 0 };

        if y_overlap > 0 || y_gap <= proximity_max {
            // 1. 无缝相贴 (0 间距)
            consider_x(other.right);
            consider_x(other.left - curr_w);

            // 2. 标准网格间距相贴 (grid_gap 间隙)
            consider_x(other.right + grid_gap);
            consider_x(other.left - curr_w - grid_gap);

            // 3. 左边缘对齐 / 右边缘对齐
            consider_x(other.left);
            consider_x(other.right - curr_w);

            // 4. 水平中心对齐
            let center_x = other.left + (other.width() - curr_w) / 2;
            consider_x(center_x);
        }
    }

    // ── Y 轴候选求解（垂直吸附与相同高度网格对齐）─────────────────
    let mut best_y: Option<(i32, i32)> = None; // (target_y, abs_delta)

    let mut consider_y = |target_y: i32| {
        let delta = (curr_y - target_y).abs();
        if delta < snap_threshold {
            if let Some((_, best_delta)) = best_y {
                if delta < best_delta {
                    best_y = Some((target_y, delta));
                }
            } else {
                best_y = Some((target_y, delta));
            }
        }
    };

    // 屏幕顶边缘与底边缘紧贴（0 缝隙）
    consider_y(work.top);
    consider_y(work.bottom - curr_h);

    // 组件间垂直对齐、相同高度网格对齐与相贴
    for other in &other_rects {
        let x_overlap = (curr_x + curr_w).min(other.right) - curr_x.max(other.left);
        let x_gap = if x_overlap < 0 { -x_overlap } else { 0 };

        if x_overlap > 0 || x_gap <= proximity_max {
            // 1. 相同高度网格顶边缘对齐（优先级最高，使并排组件高度完美齐平）
            consider_y(other.top);

            // 2. 底边缘对齐
            consider_y(other.bottom - curr_h);

            // 3. 无缝顶贴底 / 底贴顶 (0 间距)
            consider_y(other.bottom);
            consider_y(other.top - curr_h);

            // 4. 标准网格间距 (grid_gap 间隙)
            consider_y(other.bottom + grid_gap);
            consider_y(other.top - curr_h - grid_gap);

            // 5. 垂直中心对齐
            let center_y = other.top + (other.height() - curr_h) / 2;
            consider_y(center_y);
        }
    }

    // 应用最优吸附解
    if let Some((target_x, _)) = best_x {
        pos.x = target_x;
    }
    if let Some((target_y, _)) = best_y {
        pos.y = target_y;
    }
}

/// 在编辑模式下拖拽调整窗口大小时的智能网格与高度吸附 (WM_SIZING)
pub unsafe fn apply_sizing_snapping(hwnd: isize, edge: u32, rect: &mut RECT) {
    let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    let mut info: MONITORINFO = std::mem::zeroed();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    if GetMonitorInfoW(hmonitor, &mut info) == 0 {
        return;
    }

    let mut dpi_x: u32 = 96;
    let mut dpi_y: u32 = 96;
    let _ = windows_sys::Win32::UI::HiDpi::GetDpiForMonitor(hmonitor, 0, &mut dpi_x, &mut dpi_y);
    let scale = (dpi_x as f32 / 96.0).max(0.5);
    let snap_threshold = ((14.0 * scale).round() as i32).clamp(10, 24);

    let work = SnapRect {
        left: info.rcWork.left,
        top: info.rcWork.top,
        right: info.rcWork.right,
        bottom: info.rcWork.bottom,
    };

    let mut other_rects = Vec::new();
    if let Some(procs) = WND_PROCS.get() {
        if let Ok(guard) = procs.lock() {
            collect_visible_windows(hwnd, &guard, &mut other_rects);
        }
    }

    let mut curr = SnapRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };

    // 1. 调整顶边或底边时的“相同高度自动吸附”与顶/底边对齐
    let is_top = edge == WMSZ_TOP || edge == WMSZ_TOPLEFT || edge == WMSZ_TOPRIGHT;
    let is_bottom = edge == WMSZ_BOTTOM || edge == WMSZ_BOTTOMLEFT || edge == WMSZ_BOTTOMRIGHT;
    let is_left = edge == WMSZ_LEFT || edge == WMSZ_TOPLEFT || edge == WMSZ_BOTTOMLEFT;
    let is_right = edge == WMSZ_RIGHT || edge == WMSZ_TOPRIGHT || edge == WMSZ_BOTTOMRIGHT;

    if is_top {
        if (curr.top - work.top).abs() < snap_threshold {
            curr.top = work.top;
        }
        for other in &other_rects {
            // 对齐到邻近组件顶边
            if (curr.top - other.top).abs() < snap_threshold {
                curr.top = other.top;
            }
            // 自动吸附为与邻近组件“相同高度”
            let same_height_top = curr.bottom - other.height();
            if (curr.top - same_height_top).abs() < snap_threshold {
                curr.top = same_height_top;
            }
        }
    }

    if is_bottom {
        if (curr.bottom - work.bottom).abs() < snap_threshold {
            curr.bottom = work.bottom;
        }
        for other in &other_rects {
            // 对齐到邻近组件底边
            if (curr.bottom - other.bottom).abs() < snap_threshold {
                curr.bottom = other.bottom;
            }
            // 自动吸附为与邻近组件“相同高度”
            let same_height_bottom = curr.top + other.height();
            if (curr.bottom - same_height_bottom).abs() < snap_threshold {
                curr.bottom = same_height_bottom;
            }
        }
    }

    if is_left {
        if (curr.left - work.left).abs() < snap_threshold {
            curr.left = work.left;
        }
        for other in &other_rects {
            if (curr.left - other.left).abs() < snap_threshold {
                curr.left = other.left;
            }
            if (curr.left - other.right).abs() < snap_threshold {
                curr.left = other.right;
            }
        }
    }

    if is_right {
        if (curr.right - work.right).abs() < snap_threshold {
            curr.right = work.right;
        }
        for other in &other_rects {
            if (curr.right - other.right).abs() < snap_threshold {
                curr.right = other.right;
            }
            if (curr.right - other.left).abs() < snap_threshold {
                curr.right = other.left;
            }
        }
    }

    rect.left = curr.left;
    rect.top = curr.top;
    rect.right = curr.right;
    rect.bottom = curr.bottom;
}

/// 收集其他有效且可见的小组件窗口真实物理渲染矩形（通过 DWM 拓展帧获取，彻底消除隐形边框误差）
unsafe fn collect_visible_windows(
    hwnd: isize,
    guard: &std::collections::HashMap<isize, isize>,
    out: &mut Vec<SnapRect>,
) {
    for &other_hwnd in guard.keys() {
        if other_hwnd != hwnd && other_hwnd != 0 && IsWindowVisible(other_hwnd) != 0 {
            let mut dwm_rect: RECT = std::mem::zeroed();
            let hr = DwmGetWindowAttribute(
                other_hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS as u32,
                &mut dwm_rect as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            );

            if hr == 0 {
                out.push(SnapRect {
                    left: dwm_rect.left,
                    top: dwm_rect.top,
                    right: dwm_rect.right,
                    bottom: dwm_rect.bottom,
                });
            } else {
                let mut win_rect: RECT = std::mem::zeroed();
                if GetWindowRect(other_hwnd, &mut win_rect) != 0 {
                    out.push(SnapRect {
                        left: win_rect.left,
                        top: win_rect.top,
                        right: win_rect.right,
                        bottom: win_rect.bottom,
                    });
                }
            }
        }
    }
}
