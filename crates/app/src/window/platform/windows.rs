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

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_WINDOWPOSCHANGING
        && widget_core::NATIVE_EDIT_MODE.load(std::sync::atomic::Ordering::SeqCst)
    {
        unsafe {
            use windows_sys::Win32::Foundation::RECT;
            use windows_sys::Win32::Graphics::Gdi::{
                GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GetWindowRect, IsWindowVisible, WINDOWPOS,
            };
            let pos = &mut *(lparam as *mut WINDOWPOS);
            if (pos.flags & windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOMOVE) == 0 {
                let snap = 18;

                // 1. 屏幕边缘吸附与对齐 (Screen Edge Snap & Alignment)
                let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                let mut info: MONITORINFO = std::mem::zeroed();
                info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(hmonitor, &mut info) != 0 {
                    let work_rect = info.rcWork;

                    if (pos.x - work_rect.left).abs() < snap {
                        pos.x = work_rect.left;
                    } else if (work_rect.right - (pos.x + pos.cx)).abs() < snap {
                        pos.x = work_rect.right - pos.cx;
                    }

                    if (pos.y - work_rect.top).abs() < snap {
                        pos.y = work_rect.top;
                    } else if (work_rect.bottom - (pos.y + pos.cy)).abs() < snap {
                        pos.y = work_rect.bottom - pos.cy;
                    }
                }

                // 2. 组件间磁力吸附与对齐 (Component-to-Component Magnet Snap)
                if let Some(procs) = WND_PROCS.get() {
                    if let Ok(guard) = procs.lock() {
                        for &other_hwnd in guard.keys() {
                            if other_hwnd != hwnd
                                && other_hwnd != 0
                                && IsWindowVisible(other_hwnd) != 0
                            {
                                let mut other_rect: RECT = std::mem::zeroed();
                                if GetWindowRect(other_hwnd, &mut other_rect) != 0 {
                                    // 左贴右 / 右贴左 (相贴吸附)
                                    if (pos.x - other_rect.right).abs() < snap {
                                        pos.x = other_rect.right;
                                    } else if ((pos.x + pos.cx) - other_rect.left).abs() < snap {
                                        pos.x = other_rect.left - pos.cx;
                                    }

                                    // 顶贴底 / 底贴顶 (相贴吸附)
                                    if (pos.y - other_rect.bottom).abs() < snap {
                                        pos.y = other_rect.bottom;
                                    } else if ((pos.y + pos.cy) - other_rect.top).abs() < snap {
                                        pos.y = other_rect.top - pos.cy;
                                    }

                                    // 左边缘对齐 / 右边缘对齐 (对齐吸附)
                                    if (pos.x - other_rect.left).abs() < snap {
                                        pos.x = other_rect.left;
                                    } else if ((pos.x + pos.cx) - other_rect.right).abs() < snap {
                                        pos.x = other_rect.right - pos.cx;
                                    }

                                    // 顶边缘对齐 / 底边缘对齐 (对齐吸附)
                                    if (pos.y - other_rect.top).abs() < snap {
                                        pos.y = other_rect.top;
                                    } else if ((pos.y + pos.cy) - other_rect.bottom).abs() < snap {
                                        pos.y = other_rect.bottom - pos.cy;
                                    }
                                }
                            }
                        }
                    }
                }
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

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_NCHITTEST
        && !widget_core::NATIVE_EDIT_MODE.load(std::sync::atomic::Ordering::SeqCst)
    {
        if let 10..=17 = res {
            return 1; // HTCLIENT
        }
    }

    res
}

/// 应用小组件窗口的特定样式，彻底禁用原生缩放、保留置顶等特性。
///
/// 使用与 stretchly 遮罩相同的纯净 Win32 样式，不注入破坏 DirectComposition 的 Parent 窗口。
pub fn apply_plugin_window_styles(
    hwnd: isize,
    id: &str,
    config: Option<&widget_core::AppConfig>,
) -> isize {
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowLongPtrW, SetWindowLongW, SetWindowPos, GWLP_WNDPROC,
            GWL_EXSTYLE, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
            WS_BORDER, WS_CAPTION, WS_EX_CLIENTEDGE, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE, WS_POPUP,
            WS_SYSMENU, WS_THICKFRAME,
        };

        // 1. 纯净设置 Window Style（与 stretchly 遮罩完全一致）
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        SetWindowLongW(
            hwnd,
            GWL_STYLE,
            (style
                & !(WS_CAPTION as i32)
                & !(WS_THICKFRAME as i32)
                & !(WS_BORDER as i32)
                & !(WS_SYSMENU as i32))
                | WS_POPUP as i32,
        );

        // 2. 纯净设置 Extended Style（保留 TOOLWINDOW，清除无用边框）
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            (ex_style | WS_EX_TOOLWINDOW as i32)
                & !(WS_EX_CLIENTEDGE as i32)
                & !(WS_EX_WINDOWEDGE as i32),
        );

        // 3. 将 DWM 客户区玻璃拓展到全窗口（消除默认白色客户区，使 DirectComposition Alpha 直通桌面壁纸）
        #[repr(C)]
        struct MARGINS {
            cx_left_width: i32,
            cx_right_width: i32,
            cy_top_height: i32,
            cy_bottom_height: i32,
        }

        let margins = MARGINS {
            cx_left_width: -1,
            cx_right_width: -1,
            cy_top_height: -1,
            cy_bottom_height: -1,
        };
        windows_sys::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea(
            hwnd,
            &margins as *const _ as *const _,
        );

        // 4. 彻底覆盖并清除 GPUI 注入的 ACCENT_ENABLE_TRANSPARENTGRADIENT (导致 Win10/Win11 出现纯白实心背景的根源)
        #[repr(C)]
        struct AccentPolicy {
            accent_state: u32,
            accent_flags: u32,
            gradient_color: u32,
            animation_id: u32,
        }

        #[repr(C)]
        struct WindowCompositionAttributeData {
            attribute: u32,
            p_data: *mut std::ffi::c_void,
            data_size: usize,
        }

        type SetWindowCompositionAttributeFn =
            unsafe extern "system" fn(isize, *mut WindowCompositionAttributeData) -> i32;

        let user32 = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(
            windows_sys::core::w!("user32.dll"),
        );
        if user32 != 0 {
            let func_ptr = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                user32,
                windows_sys::core::s!("SetWindowCompositionAttribute"),
            );
            if let Some(func) = func_ptr {
                let set_fn: SetWindowCompositionAttributeFn = std::mem::transmute(func);
                let mut accent = AccentPolicy {
                    accent_state: 0, // ACCENT_DISABLED (彻底清除 GPUI 的白色渐变残影)
                    accent_flags: 0,
                    gradient_color: 0,
                    animation_id: 0,
                };
                let mut data = WindowCompositionAttributeData {
                    attribute: 19, // WCA_ACCENT_POLICY
                    p_data: &mut accent as *mut _ as *mut std::ffi::c_void,
                    data_size: std::mem::size_of::<AccentPolicy>(),
                };
                set_fn(hwnd, &mut data);
            }
        }

        SetWindowPos(
            hwnd,
            0,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
        );

        // 5. 注入自定义窗口过程（用于拦截 WM_ERASEBKGND、组件对齐吸附与编辑模式拖拽）
        let old_proc = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, plugin_wnd_proc as *const () as isize);
        if old_proc != 0 {
            let procs = WND_PROCS.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
            procs.lock().unwrap().insert(hwnd, old_proc);
        }
    }

    // 4. 恢复独立设置（置顶和鼠标穿透）
    if let Some(cfg) = config {
        if let Some(plugin_cfg) = cfg.plugins.get(id) {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    SetWindowPos, HWND_BOTTOM, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
                };
                let insert_after = if plugin_cfg.always_on_top {
                    HWND_TOPMOST
                } else {
                    HWND_BOTTOM
                };
                SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);

                if plugin_cfg.mouse_passthrough {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{
                        GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_LAYERED,
                        WS_EX_TRANSPARENT,
                    };
                    let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                    SetWindowLongW(
                        hwnd,
                        GWL_EXSTYLE,
                        style | WS_EX_TRANSPARENT as i32 | WS_EX_LAYERED as i32,
                    );
                }
            }
        }
    }

    0
}

/// 移除小组件窗口的自定义样式，还原原生的窗口回调过程。
pub fn cleanup_plugin_window_styles(hwnd: isize) {
    if hwnd == 0 {
        return;
    }
    let procs = WND_PROCS.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    if let Some(old_proc) = procs.lock().unwrap().remove(&hwnd) {
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC,
                old_proc,
            );
        }
        println!(
            "[cleanup_plugin_window_styles] 已还原 HWND {} 的 WndProc",
            hwnd
        );
    }
}

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
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_BOTTOM, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
    };
    for (id, (_, hwnd, _)) in widget_windows {
        if *hwnd == 0 {
            continue;
        }
        unsafe {
            let insert_after = if always_on_top {
                HWND_TOPMOST
            } else {
                HWND_BOTTOM
            };
            SetWindowPos(*hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
        println!("[WindowManager] 插件 {} 置顶: {}", id, always_on_top);
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
                log_x = rect.left as f32 / actual_scale;
                log_y = rect.top as f32 / actual_scale;
                log_w = (rect.right - rect.left) as f32 / actual_scale;
                log_h = (rect.bottom - rect.top) as f32 / actual_scale;
            } else {
                if GetWindowRect(hwnd, &mut rect) != 0 {
                    log_x = rect.left as f32 / actual_scale;
                    log_y = rect.top as f32 / actual_scale;
                    log_w = (rect.right - rect.left) as f32 / actual_scale;
                    log_h = (rect.bottom - rect.top) as f32 / actual_scale;
                }
            }

            let mut phys_rect: RECT = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut phys_rect) != 0 {
                phys_x = phys_rect.left;
                phys_y = phys_rect.top;
                phys_w = phys_rect.right - phys_rect.left;
                phys_h = phys_rect.bottom - phys_rect.top;
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
