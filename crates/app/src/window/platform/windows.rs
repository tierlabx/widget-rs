use std::sync::{Mutex, OnceLock};

pub static WND_PROCS: OnceLock<Mutex<std::collections::HashMap<isize, isize>>> = OnceLock::new();

/// 插件窗口的消息处理回调。
///
/// 负责处理边界吸附、防止失去焦点以及鼠标交互穿透等特定需求。
pub unsafe extern "system" fn plugin_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let old_proc = {
        let procs = WND_PROCS.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        let guard = procs.lock().unwrap();
        *guard.get(&hwnd).unwrap_or(&0)
    };

    if old_proc == 0 {
        return windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(
            hwnd, msg, wparam, lparam,
        );
    }

    let old_proc_fn: unsafe extern "system" fn(isize, u32, usize, isize) -> isize =
        std::mem::transmute(old_proc);

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_NCCALCSIZE && wparam != 0 {
        // 彻底消除 Windows 10/11 的 Invisible resize border 带来的隐形缩放边框与吸附空隙
        return 0;
    }

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_SIZING
        && widget_core::NATIVE_EDIT_MODE.load(std::sync::atomic::Ordering::SeqCst)
    {
        unsafe {
            use windows_sys::Win32::Foundation::RECT;
            let rect = &mut *(lparam as *mut RECT);
            super::snap::apply_sizing_snapping(hwnd, wparam as u32, rect);
            return 1;
        }
    }

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_WINDOWPOSCHANGING {
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::WINDOWPOS;
            let pos = &mut *(lparam as *mut WINDOWPOS);

            // 【核心修复 - 独立聚焦】阻断共享 Progman Owner 的组件间“组式 Z 序联动置前”
            // ─────────────────────────────────────────────────────────────────────────────
            // 根因：所有小组件都挂载到 Progman（Win+D 桌面常驻能力，见 styles.rs 第 5 步），
            // 同 Owner 的窗口构成 Z 序联动组：激活任意一个，Windows 会把同组兄弟窗口全部
            // 拉到前台，表现为“点击一个小组件，其他全部聚焦”。
            // 修复：在 WM_WINDOWPOSCHANGING 中，对“非激活窗口的被动 Z 序变更”追加
            // SWP_NOZORDER，仅放行被点击窗口自身的置前与显式置顶操作，
            // 保持各小组件聚焦独立（已通过 .tmp-ztest/ztest.cs 复现并验证）。
            // ─────────────────────────────────────────────────────────────────────────────
            {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetForegroundWindow, GetWindowLongW, GWL_EXSTYLE, HWND_NOTOPMOST, HWND_TOPMOST,
                    SWP_NOZORDER, WS_EX_TOPMOST,
                };
                let z_change = (pos.flags & SWP_NOZORDER) == 0;
                let topmost_op =
                    pos.hwndInsertAfter == HWND_TOPMOST || pos.hwndInsertAfter == HWND_NOTOPMOST;
                let is_topmost = (GetWindowLongW(hwnd, GWL_EXSTYLE) & WS_EX_TOPMOST as i32) != 0;
                let self_active = GetActiveWindow() == hwnd || GetForegroundWindow() == hwnd;
                let explicit_allowed =
                    widget_core::ALLOW_EXPLICIT_ZORDER.load(std::sync::atomic::Ordering::SeqCst);
                if z_change && !topmost_op && !self_active && !is_topmost && !explicit_allowed {
                    pos.flags |= SWP_NOZORDER;
                }
            }

            // 编辑模式下的智能吸附（原有逻辑）
            if widget_core::NATIVE_EDIT_MODE.load(std::sync::atomic::Ordering::SeqCst)
                && (pos.flags & windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOMOVE) == 0
            {
                super::snap::apply_window_snapping(hwnd, pos);
            }
        }
    }

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_ERASEBKGND {
        return 1; // 拦截默认背景擦除，彻底杜绝 Windows 默认白色画刷填充
    }

    // 移除编辑模式外的多余窗口过程拦截，仅在编辑模式或吸附时使用
    let res = windows_sys::Win32::UI::WindowsAndMessaging::CallWindowProcW(
        Some(old_proc_fn),
        hwnd,
        msg,
        wparam,
        lparam,
    );

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_NCHITTEST {
        if widget_core::NATIVE_EDIT_MODE.load(std::sync::atomic::Ordering::SeqCst) {
            unsafe {
                use windows_sys::Win32::Foundation::RECT;
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetWindowRect, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTLEFT, HTRIGHT, HTTOP,
                    HTTOPLEFT, HTTOPRIGHT,
                };
                let mut rect: RECT = std::mem::zeroed();
                if GetWindowRect(hwnd, &mut rect) != 0 {
                    let x = (lparam & 0xFFFF) as i16 as i32;
                    let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                    let border = 7;

                    let is_left = x >= rect.left && x < rect.left + border;
                    let is_right = x < rect.right && x >= rect.right - border;
                    let is_top = y >= rect.top && y < rect.top + border;
                    let is_bottom = y < rect.bottom && y >= rect.bottom - border;

                    if is_top && is_left {
                        return HTTOPLEFT as isize;
                    }
                    if is_top && is_right {
                        return HTTOPRIGHT as isize;
                    }
                    if is_bottom && is_left {
                        return HTBOTTOMLEFT as isize;
                    }
                    if is_bottom && is_right {
                        return HTBOTTOMRIGHT as isize;
                    }
                    if is_left {
                        return HTLEFT as isize;
                    }
                    if is_right {
                        return HTRIGHT as isize;
                    }
                    if is_top {
                        return HTTOP as isize;
                    }
                    if is_bottom {
                        return HTBOTTOM as isize;
                    }
                }
            }
        } else if let 10..=17 = res {
            return 1; // HTCLIENT
        }
    }

    res
}

pub use super::styles::{apply_plugin_window_styles, cleanup_plugin_window_styles};

/// 在 Windows 环境下，通过底层 API 切换主窗口的显示与隐藏状态。
pub fn toggle_main_window_win32(main_hwnd: isize, is_visible_current: bool) -> bool {
    let hwnd = main_hwnd;
    if hwnd == 0 {
        return is_visible_current;
    }
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE,
            SW_SHOW,
        };
        let is_win_visible = IsWindowVisible(hwnd) != 0;
        let is_minimized = IsIconic(hwnd) != 0;
        let next_visible = !is_win_visible || is_minimized;
        if next_visible {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            } else {
                ShowWindow(hwnd, SW_SHOW);
            }
            SetForegroundWindow(hwnd);
        } else {
            ShowWindow(hwnd, SW_HIDE);
        }
        println!("[WindowManager] 主窗口切换: is_visible = {}", next_visible);
        next_visible
    }
}

/// 批量应用所有小组件的置顶/垫底设置。
pub fn apply_always_on_top(
    widget_windows: &std::collections::HashMap<&'static str, (gpui::AnyWindowHandle, isize, isize)>,
    always_on_top: bool,
) {
    for (id, (_, hwnd, _)) in widget_windows {
        if *hwnd != 0 {
            widget_core::set_window_always_on_top(*hwnd, always_on_top);
            println!("[WindowManager] 插件 {} 置顶: {}", id, always_on_top);
        }
    }
}

/// 控制特定插件窗口的可见性。
pub fn show_plugin_window(hwnd: isize, visible: bool) {
    if hwnd == 0 {
        return;
    }
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOW};
        if visible {
            ShowWindow(hwnd, SW_SHOW);
        } else {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
    println!("[WindowManager] HWND {} 可见性: {}", hwnd, visible);
}

/// 销毁给定的 HWND 窗口句柄。
pub fn destroy_window(hwnd: isize) {
    if hwnd != 0 {
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
        }
    }
}

/// 获取指定 HWND 的窗口边界。
///
/// 返回值依次为：逻辑 X, 逻辑 Y, 逻辑宽, 逻辑高, 实际缩放比, 物理 X, 物理 Y, 物理宽, 物理高。
pub fn get_window_bounds(hwnd: isize, scale: f32) -> (f32, f32, f32, f32, f32, i32, i32, i32, i32) {
    let mut actual_scale = scale;
    let mut log_x = 0.0;
    let mut log_y = 0.0;
    let mut log_w = 0.0;
    let mut log_h = 0.0;
    let mut phys_x = 0;
    let mut phys_y = 0;
    let mut phys_w = 0;
    let mut phys_h = 0;

    if hwnd != 0 {
        unsafe {
            use windows_sys::Win32::Foundation::RECT;
            use windows_sys::Win32::Graphics::Dwm::{
                DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS,
            };
            use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
            use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect;

            let dpi = GetDpiForWindow(hwnd);
            actual_scale = if dpi == 0 { scale } else { dpi as f32 / 96.0 };

            let mut rect: RECT = std::mem::zeroed();
            let hr = DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS as u32,
                &mut rect as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            );

            if hr == 0 {
                // 优先使用 DWM 真实视觉物理边界（彻底消除 Windows 10/11 系统的 Invisible resize border 误差）
                phys_x = rect.left;
                phys_y = rect.top;
                phys_w = rect.right - rect.left;
                phys_h = rect.bottom - rect.top;
                log_x = rect.left as f32 / actual_scale;
                log_y = rect.top as f32 / actual_scale;
                log_w = phys_w as f32 / actual_scale;
                log_h = phys_h as f32 / actual_scale;
            } else {
                let mut phys_rect: RECT = std::mem::zeroed();
                if GetWindowRect(hwnd, &mut phys_rect) != 0 {
                    phys_x = phys_rect.left;
                    phys_y = phys_rect.top;
                    phys_w = phys_rect.right - phys_rect.left;
                    phys_h = phys_rect.bottom - phys_rect.top;
                    log_x = phys_rect.left as f32 / actual_scale;
                    log_y = phys_rect.top as f32 / actual_scale;
                    log_w = phys_w as f32 / actual_scale;
                    log_h = phys_h as f32 / actual_scale;
                }
            }
        }
    }
    (
        log_x,
        log_y,
        log_w,
        log_h,
        actual_scale,
        phys_x,
        phys_y,
        phys_w,
        phys_h,
    )
}
