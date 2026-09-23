use gpui::Window;

/// 触发窗口拖拽
pub fn start_window_drag(window: &mut Window) {
    use raw_window_handle::HasWindowHandle;
    if let Ok(handle) = HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
            unsafe {
                windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                    h.hwnd.get(),
                    windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN,
                    windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize,
                    0,
                );
            }
        }
    }
}

/// 更新窗口的编辑模式样式（边框可拖拽调整大小）
pub fn update_window_edit_mode(window: &mut Window, is_edit_mode: bool) {
    use raw_window_handle::HasWindowHandle;
    if let Ok(handle) = HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
            let hwnd = h.hwnd.get();
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_STYLE, SWP_FRAMECHANGED,
                    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_THICKFRAME,
                };
                let style = GetWindowLongW(hwnd, GWL_STYLE);
                if is_edit_mode {
                    if (style & WS_THICKFRAME as i32) == 0 {
                        SetWindowLongW(hwnd, GWL_STYLE, style | WS_THICKFRAME as i32);
                        SetWindowPos(
                            hwnd,
                            0,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                        );
                    }
                } else if (style & WS_THICKFRAME as i32) != 0 {
                    SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_THICKFRAME as i32));
                    SetWindowPos(
                        hwnd,
                        0,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                    );
                }
            }
        }
    }
}

/// 主动触发进程闲置堆内存归还给操作系统内核
static MEMORY_TRIM_FN: std::sync::RwLock<Option<fn()>> = std::sync::RwLock::new(None);

/// 注册底层内存整理回调函数（由启用 mimalloc 的主程序在启动时挂载）
pub fn register_memory_trim_fn(f: fn()) {
    if let Ok(mut lock) = MEMORY_TRIM_FN.write() {
        *lock = Some(f);
    }
}

/// 主动触发进程闲置堆内存归还给操作系统内核
///
/// 通过主应用注册的 `mi_collect(true)` 将已释放但缓存在内存池中的空闲页安全归还给 OS，
/// 真实减少进程占用的物理工作集，同时避免了 Win32 `EmptyWorkingSet` 导致的剧烈缺页抖动。
pub fn trim_process_memory() {
    if let Ok(lock) = MEMORY_TRIM_FN.read() {
        if let Some(trim_fn) = *lock {
            trim_fn();
        }
    }
}

use std::sync::atomic::{AtomicBool, Ordering};

/// 允许显式 Z 序/置顶变更的全局标记，防止窗口过程中的组联动拦截器误判
pub static ALLOW_EXPLICIT_ZORDER: AtomicBool = AtomicBool::new(false);

/// 显示器配置已变更的全局标记（接入/拔出显示器、分辨率/DPI 变更）
///
/// 由 `plugin_wnd_proc` 在收到 `WM_DISPLAYCHANGE` 时设为 `true`，
/// 由 `spawn_tray_polling_task` 在下次轮询时读取并清零，
/// 触发 `WindowManager::reposition_all_to_valid_screens` 自动将离屏插件归位。
pub static DISPLAY_CHANGED: AtomicBool = AtomicBool::new(false);

/// 设置或取消窗口的系统级置顶状态（Always on Top）
///
/// 遵循 Win32 窗口体系规范：
/// - 置顶时：将小组件解绑 Progman 桌面宿主（GWLP_HWNDPARENT = 0），并赋予 HWND_TOPMOST 成为全局悬浮置顶窗。
///   根因：Win32 规定非置顶窗口（Progman）不能作为置顶窗口的宿主，若不解绑会被锁死在底层桌面而被编辑器等遮挡。
/// - 取消置顶时：使用 HWND_NOTOPMOST 撤销置顶，并重新挂载回 Progman 桌面窗口以恢复 Win+D 桌面常驻特性。
pub fn set_window_always_on_top(hwnd: isize, always_on_top: bool) {
    if hwnd == 0 {
        return;
    }
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            BringWindowToTop, FindWindowW, SetWindowLongPtrW, SetWindowPos, GWLP_HWNDPARENT,
            HWND_NOTOPMOST, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SWP_SHOWWINDOW,
        };

        // 显式允许本次 Z 序变更，避免窗口过程拦截
        ALLOW_EXPLICIT_ZORDER.store(true, Ordering::SeqCst);

        if always_on_top {
            // 1. 置顶窗口脱离 Progman 宿主
            SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, 0);

            // 2. 赋予全局 TOPMOST 状态并刷新显示
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );

            // 3. 立即将窗口提升至顶层，无需用户手动再点击一次
            BringWindowToTop(hwnd);
        } else {
            // 1. 撤销全局 TOPMOST 状态
            SetWindowPos(
                hwnd,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );

            // 2. 重新挂载到 Progman 桌面窗口（恢复 Win+D 桌面常驻能力）
            let progman = FindWindowW(windows_sys::core::w!("Progman"), std::ptr::null());
            if progman != 0 {
                SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, progman);
            }
        }

        ALLOW_EXPLICIT_ZORDER.store(false, Ordering::SeqCst);
    }
}

/// 设置或取消窗口的鼠标点击穿透（WS_EX_TRANSPARENT）
pub fn set_window_mouse_passthrough(hwnd: isize, passthrough: bool) {
    if hwnd == 0 {
        return;
    }
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_LAYERED, WS_EX_TRANSPARENT,
        };
        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if passthrough {
            SetWindowLongW(
                hwnd,
                GWL_EXSTYLE,
                style | WS_EX_TRANSPARENT as i32 | WS_EX_LAYERED as i32,
            );
        } else {
            SetWindowLongW(hwnd, GWL_EXSTYLE, style & !(WS_EX_TRANSPARENT as i32));
        }

        // 必须调用 SetWindowPos 并带有 SWP_FRAMECHANGED 才能让窗口样式立即生效
        SetWindowPos(
            hwnd,
            0,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
        );
    }
}

/// 控制特定窗口的显示与隐藏（基于 HWND）
pub fn show_window_hwnd(hwnd: isize, visible: bool) {
    if hwnd == 0 {
        return;
    }
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOW};
        if visible {
            ShowWindow(hwnd, SW_SHOW);
        } else {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}
