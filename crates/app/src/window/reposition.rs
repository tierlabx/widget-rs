use gpui::App;
use std::collections::HashMap;

/// 将所有离屏或超出工作区的插件窗口归位到最近的有效显示器工作区内
///
/// 在收到 `WM_DISPLAYCHANGE`（显示器接入/拔出/分辨率变更）后调用，
/// 防止插件永久隐藏在不再存在的副屏坐标处。
///
/// 策略：
/// - 若窗口仍在某显示器工作区内 → 无需操作
/// - 若窗口完全离屏（副屏已拔除）→ `find_best_monitor` 回退到主屏，`clamp_to_work_area` 归位
pub fn reposition_all_to_valid_screens(
    widget_windows: &HashMap<String, (gpui::AnyWindowHandle, isize, isize)>,
    cx: &mut App,
) {
    let monitors = widget_core::enumerate_monitors();
    if monitors.is_empty() {
        return;
    }

    let mut config = cx
        .try_global::<widget_core::AppConfig>()
        .cloned()
        .unwrap_or_default();
    let mut any_moved = false;

    for (id, (_, hwnd, _)) in widget_windows {
        if *hwnd == 0 {
            continue;
        }

        let Some((px, py, pw, ph)) =
            crate::window::platform::windows::get_hwnd_physical_rect(*hwnd)
        else {
            continue;
        };

        let Some(target_monitor) = widget_core::find_best_monitor(&monitors, px, py, pw, ph) else {
            continue;
        };

        let (nx, ny, nw, nh) = widget_core::clamp_to_work_area(target_monitor, px, py, pw, ph);

        if nx == px && ny == py && nw == pw && nh == ph {
            // 已经在工作区内，无需移动
            continue;
        }

        // 窗口位置需要修正（离屏或超出工作区）
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                *hwnd,
                0,
                nx,
                ny,
                nw,
                nh,
                windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                    | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
            );
        }

        // 同步更新配置中的物理坐标，确保下次重启时位置正确
        let entry =
            config
                .plugins
                .entry(id.to_string())
                .or_insert_with(|| widget_core::PluginConfig {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                    scale: target_monitor.scale_factor,
                    phys_x: 0,
                    phys_y: 0,
                    phys_w: 0,
                    phys_h: 0,
                    always_on_top: false,
                    mouse_passthrough: false,
                    pinned_to_desktop: false,
                    loaded: true,
                    enabled: true,
                });
        entry.phys_x = nx;
        entry.phys_y = ny;
        entry.phys_w = nw;
        entry.phys_h = nh;
        entry.scale = target_monitor.scale_factor;
        entry.x = nx as f32 / target_monitor.scale_factor;
        entry.y = ny as f32 / target_monitor.scale_factor;
        entry.width = nw as f32 / target_monitor.scale_factor;
        entry.height = nh as f32 / target_monitor.scale_factor;

        any_moved = true;
        println!(
            "[WindowManager] 显示器变更自愈：插件 {} 已归位到 ({}, {}) {}x{}",
            id, nx, ny, nw, nh
        );
    }

    if any_moved {
        cx.set_global(config);
    }
}
