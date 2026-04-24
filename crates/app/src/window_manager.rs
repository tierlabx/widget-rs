use gpui::*;
use widget_ui::main_window::MainWindow;
use widget_core::{AppConfig, PluginConfig};
use std::collections::HashMap;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ShowWindow, GetWindowRect,
};
use windows_sys::Win32::Foundation::RECT;
use crate::store::Store;

pub struct WindowManager {
    pub main_window: Option<WindowHandle<gpui_component::Root>>,
    /// 主窗口的 Win32 HWND（提取后存储，避免 toggle 时嵌套借用）
    pub main_hwnd: isize,
    /// 注册的插件窗口：id -> (窗口句柄, HWND)
    pub widget_windows: HashMap<&'static str, (AnyWindowHandle, isize)>,
    pub is_visible: bool,
}

impl Global for WindowManager {}

impl WindowManager {
    pub fn init(cx: &mut App) {
        cx.set_global(widget_core::UIState { is_visible: true, is_edit_mode: false, plugin_visibility: std::collections::HashMap::new() });
        cx.set_global(Self {
            main_window: None,
            main_hwnd: 0,
            widget_windows: HashMap::new(),
            is_visible: true,
        });
        
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: None,
                appears_transparent: true,
                traffic_light_position: None,
            }),
            window_background: WindowBackgroundAppearance::Transparent,
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1200.0), px(800.0)), cx))),
            ..Default::default()
        };

        let window = cx.open_window(options, |window, cx| {
            let view = cx.new(|_| MainWindow::new());
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        }).unwrap();
        
        cx.update_global::<Self, _>(|wm, _cx| {
            wm.main_window = Some(window);
        });
    }

    /// 注册插件窗口，同时记录 HWND 以便后续读取位置
    pub fn register_widget_window(&mut self, id: &'static str, handle: AnyWindowHandle) {
        // 尝试从窗口句柄中提取 HWND
        let hwnd = Self::extract_hwnd(&handle);
        self.widget_windows.insert(id, (handle, hwnd));
    }

    /// 从 AnyWindowHandle 中提取 Win32 HWND
    fn extract_hwnd(handle: &AnyWindowHandle) -> isize {
        // AnyWindowHandle 无法直接在此安全上下文中读取句柄，
        // 我们通过存储插件窗口时用额外调用来获取。
        // 这里先存 0，在插件首次渲染时会通过 set_hwnd 更新。
        let _ = handle;
        0
    }

    /// 在插件首次渲染后，通过插件 ID 更新已记录的 HWND
    #[allow(dead_code)]
    pub fn set_hwnd(&mut self, id: &'static str, hwnd: isize) {
        if let Some(entry) = self.widget_windows.get_mut(id) {
            entry.1 = hwnd;
        }
    }

    /// 读取所有已注册插件的当前窗口位置并保存到配置文件
    pub fn save_all_plugin_bounds(&self, cx: &mut App, store: &Store) {
        let mut config = cx
            .try_global::<AppConfig>()
            .cloned()
            .unwrap_or_default();

        for (id, (_handle, hwnd)) in &self.widget_windows {
            if *hwnd == 0 {
                continue;
            }
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            let ok = unsafe { GetWindowRect(*hwnd, &mut rect) };
            if ok != 0 {
                config.plugins.insert(
                    id.to_string(),
                    PluginConfig {
                        x: rect.left as f32,
                        y: rect.top as f32,
                        width: (rect.right - rect.left) as f32,
                        height: (rect.bottom - rect.top) as f32,
                    },
                );
                println!(
                    "[WindowManager] 保存插件 {} 位置: ({}, {}) {}x{}",
                    id, rect.left, rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top
                );
            }
        }

        // 写回全局状态
        cx.set_global(config.clone());
        store.save_config(&config);
    }

    /// 获取插件的 HWND（用于在异步上下文中安全操作，避免嵌套借用）
    pub fn get_plugin_hwnd(&self, plugin_id: &str) -> isize {
        self.widget_windows.iter()
            .find(|(k, _)| **k == plugin_id)
            .map(|(_, (_, hwnd))| *hwnd)
            .unwrap_or(0)
    }

    /// 通过 HWND 直接控制插件窗口显示/隐藏（不借用 cx，可在 update_global 外调用）
    pub fn show_plugin_window(hwnd: isize, visible: bool) {
        if hwnd == 0 { return; }
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{SW_SHOW, SW_HIDE};
            if visible {
                ShowWindow(hwnd, SW_SHOW);
            } else {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
        println!("[WindowManager] HWND {} 可见性: {}", hwnd, visible);
    }

    /// 应用"始终置顶"设置到所有插件窗口
    pub fn apply_always_on_top(&self, always_on_top: bool) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_TOPMOST, HWND_NOTOPMOST,
            SWP_NOMOVE, SWP_NOSIZE,
        };
        for (id, (_, hwnd)) in &self.widget_windows {
            if *hwnd == 0 { continue; }
            unsafe {
                let insert_after = if always_on_top { HWND_TOPMOST } else { HWND_NOTOPMOST };
                SetWindowPos(*hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            }
            println!("[WindowManager] 插件 {} 置顶: {}", id, always_on_top);
        }
    }

    /// 应用"鼠标穿透"设置到所有插件窗口
    pub fn apply_mouse_passthrough(&self, passthrough: bool) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowLongW, GWL_EXSTYLE,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TRANSPARENT;
        for (id, (_, hwnd)) in &self.widget_windows {
            if *hwnd == 0 { continue; }
            unsafe {
                let ex_style = GetWindowLongW(*hwnd, GWL_EXSTYLE);
                let new_style = if passthrough {
                    ex_style | WS_EX_TRANSPARENT as i32
                } else {
                    ex_style & !(WS_EX_TRANSPARENT as i32)
                };
                SetWindowLongW(*hwnd, GWL_EXSTYLE, new_style);
            }
            println!("[WindowManager] 插件 {} 鼠标穿透: {}", id, passthrough);
        }
    }

    /// 获取主窗口的 HWND（避免在 update_global 闭包内嵌套调用）
    pub fn get_main_hwnd(&self) -> isize {
        // 只能通过已存储的值，或者在外部通过 window.update 读取
        // 这里返回 0 占位，主窗口 HWND 通过 toggle_main_window_win32 外部传入
        0
    }

    /// 通过 Win32 API 切换主窗口可见性，不持有 cx borrow
    /// 返回：(next_visible, hwnd)
    pub fn compute_next_visible(&mut self) -> bool {
        !self.is_visible
    }

    /// 更新 is_visible 字段
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    /// 切换主窗口显示/隐藏，纯 Win32 实现，不借用 cx（必须先调用 set_main_hwnd）
    pub fn toggle_main_window_win32(&mut self) -> bool {
        let hwnd = self.main_hwnd;
        if hwnd == 0 {
            return self.is_visible;
        }
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                IsWindowVisible, IsIconic, IsZoomed,
                ShowWindow, SW_RESTORE, SW_SHOW, SW_HIDE, SetForegroundWindow
            };
            let is_win_visible = IsWindowVisible(hwnd) != 0;
            let is_minimized = IsIconic(hwnd) != 0;
            let next_visible = !(is_win_visible && !is_minimized);
            self.is_visible = next_visible;
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

    /// 兼容旧接口（托盘事件中使用），已迁移为 toggle_main_window_win32
    pub fn toggle_main_window(&mut self, _cx: &mut App) {
        self.toggle_main_window_win32();
    }
}
