use std::sync::Mutex;
use widget_core::AppConfig;

use super::windows::{plugin_wnd_proc, WND_PROCS};

/// 应用小组件窗口的特定样式，彻底禁用原生缩放、保留置顶等特性。
///
/// 使用与 stretchly 遮罩相同的纯净 Win32 样式，不注入破坏 DirectComposition 的 Parent 窗口。
pub fn apply_plugin_window_styles(hwnd: isize, id: &str, config: Option<&AppConfig>) -> isize {
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowLongPtrW, SetWindowLongW, SetWindowPos, GWLP_WNDPROC,
            GWL_EXSTYLE, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
            WS_BORDER, WS_CAPTION, WS_EX_CLIENTEDGE, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE, WS_POPUP,
            WS_SYSMENU, WS_THICKFRAME,
        };

        // 1. 纯净设置 Window Style
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
        #[allow(non_camel_case_types)]
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

        // 4. 彻底覆盖并清除 GPUI 注入的 ACCENT_ENABLE_TRANSPARENTGRADIENT
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
                    accent_state: 0,
                    accent_flags: 0,
                    gradient_color: 0,
                    animation_id: 0,
                };
                let mut data = WindowCompositionAttributeData {
                    attribute: 19,
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

        // 5. 注入自定义窗口过程
        let old_proc = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, plugin_wnd_proc as *const () as isize);
        if old_proc != 0 {
            let procs = WND_PROCS.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
            procs.lock().unwrap().insert(hwnd, old_proc);
        }
    }

    // 6. 恢复独立设置（置顶和鼠标穿透）
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
