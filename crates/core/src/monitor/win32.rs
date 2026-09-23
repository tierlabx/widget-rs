use super::types::{MonitorInfo, Rect};

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
