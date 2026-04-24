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
    /// 注册的插件窗口：id -> (窗口句柄, HWND)
    pub widget_windows: HashMap<&'static str, (AnyWindowHandle, isize)>,
    pub is_visible: bool,
}

impl Global for WindowManager {}

impl WindowManager {
    pub fn init(cx: &mut App) {
        cx.set_global(widget_core::UIState { is_visible: true, is_edit_mode: false });
        cx.set_global(Self {
            main_window: None,
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

    pub fn toggle_main_window(&mut self, cx: &mut App) {
        let mut next_visible = !self.is_visible;
        
        if let Some(window) = &self.main_window {
            window.update(cx, |_, window, _cx| {
                if let Ok(handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(h) = handle.as_raw() {
                        unsafe {
                            let hwnd = h.hwnd.get() as isize;
                            use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindowVisible, IsIconic};
                            let is_win_visible = IsWindowVisible(hwnd) != 0;
                            let is_minimized = IsIconic(hwnd) != 0;
                            
                            if is_win_visible && !is_minimized {
                                next_visible = false;
                            } else {
                                next_visible = true;
                            }
                        }
                    }
                }
            }).ok();
        }

        self.is_visible = next_visible;
        let is_visible = self.is_visible;
        
        cx.update_global::<widget_core::UIState, _>(|state, _| {
            state.is_visible = is_visible;
        });
        println!("切换主窗口可见性: is_visible = {}", is_visible);
        
        if let Some(window) = &self.main_window {
            window.update(cx, |_, window, cx| {
                if let Ok(handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(h) = handle.as_raw() {
                        unsafe {
                            let hwnd = h.hwnd.get() as isize;
                            use windows_sys::Win32::UI::WindowsAndMessaging::{IsIconic, SW_RESTORE, SW_SHOW, SW_HIDE, SetForegroundWindow};
                            if is_visible {
                                if IsIconic(hwnd) != 0 {
                                    ShowWindow(hwnd, SW_RESTORE);
                                } else {
                                    ShowWindow(hwnd, SW_SHOW);
                                }
                                SetForegroundWindow(hwnd);
                            } else {
                                ShowWindow(hwnd, SW_HIDE);
                            }
                        }
                    }
                }
                cx.notify();
            }).ok();
        }
    }
}
