use gpui::*;
use std::collections::HashMap;
use widget_core::AppConfig;
use widget_ui::main_window::MainWindow;

use crate::store::Store;
use windows_sys::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, ShowWindow, GWLP_HWNDPARENT};

/// 窗口管理器
///
/// 负责全局窗口的状态管理、插件窗口位置的读写、可见性控制、以及应用 Windows 平台相关的窗口效果（如置顶、鼠标穿透、防最小化等）。
pub struct WindowManager {
    /// 主窗口的句柄引用，可能尚未初始化
    pub main_window: Option<WindowHandle<gpui_component::Root>>,
    /// 主窗口的 Win32 HWND（提取后单独存储，避免后续操作时产生不必要的生命周期或借用嵌套）
    pub main_hwnd: isize,
    /// 注册的所有插件窗口：插件 ID 映射到 (窗口泛型句柄, Win32 HWND, Owner HWND)
    pub widget_windows: HashMap<&'static str, (AnyWindowHandle, isize, isize)>,
    /// 全局应用是否可见的状态标志
    pub is_visible: bool,
}

impl Global for WindowManager {}

impl WindowManager {
    /// 初始化窗口管理器和主窗口
    ///
    /// - 注册全局 UI 状态
    /// - 实例化并打开主窗口
    /// - 将 WindowManager 自身保存至应用的全局状态中
    pub fn init(cx: &mut App) {
        let mut plugin_loaded = std::collections::HashMap::new();
        let mut plugin_enabled = std::collections::HashMap::new();

        if let Some(config) = cx.try_global::<AppConfig>() {
            for (id, plugin_cfg) in &config.plugins {
                plugin_loaded.insert(id.clone(), plugin_cfg.loaded);
                plugin_enabled.insert(id.clone(), plugin_cfg.enabled);
            }
        }

        cx.set_global(widget_core::UIState {
            is_visible: true,
            is_edit_mode: false,
            plugin_loaded,
            plugin_enabled,
        });
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
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1200.0), px(800.0)),
                cx,
            ))),
            ..Default::default()
        };

        let window = cx
            .open_window(options, |window, cx| {
                let view = cx.new(|_| MainWindow::new());
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })
            .unwrap();

        cx.update_global::<Self, _>(|wm, _cx| {
            wm.main_window = Some(window);
        });
    }

    /// 注册插件窗口
    ///
    /// 将插件窗口记录到 `widget_windows` 字典中。初始时如果无法直接读取 HWND，则记录为 0。
    pub fn register_widget_window(&mut self, id: &'static str, handle: AnyWindowHandle) {
        // 尝试从窗口句柄中提取 HWND，默认先给 0
        let hwnd = Self::extract_hwnd(&handle);
        self.widget_windows.insert(id, (handle, hwnd, 0));
    }

    /// 移除插件窗口记录，同时销毁关联的隐藏 Owner 窗口
    pub fn remove_widget_window(&mut self, id: &str) {
        if let Some((_, _, owner_hwnd)) = self.widget_windows.remove(id) {
            if owner_hwnd != 0 {
                unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(owner_hwnd);
                }
                println!(
                    "[WindowManager] 已销毁插件 {} 的隐藏 Owner HWND: {}",
                    id, owner_hwnd
                );
            }
        }
    }

    /// 从 `AnyWindowHandle` 中尝试提取 Win32 HWND
    ///
    /// 注意：`AnyWindowHandle` 无法在此处直接安全地同步读取句柄。
    /// 这里的处理逻辑是先返回 0 作为占位符，等到插件窗口首次实际渲染或异步流程中，
    /// 再调用 `set_hwnd` 进行真实 HWND 的更新。
    fn extract_hwnd(handle: &AnyWindowHandle) -> isize {
        let _ = handle;
        0
    }

    /// 更新已注册的插件 HWND
    ///
    /// 此方法通常在插件窗口渲染完成、可以通过底层 API 拿到真正系统句柄后调用。
    #[allow(dead_code)]
    pub fn set_hwnd(&mut self, id: &'static str, hwnd: isize) {
        if let Some(entry) = self.widget_windows.get_mut(id) {
            entry.1 = hwnd;
        }
    }

    /// 保存所有插件的当前屏幕位置和尺寸到配置文件
    ///
    /// 通过 GPUI 内部方法读取逻辑位置（DIPs），确保位置在不同 DPI 缩放下依然准确。
    pub fn save_all_plugin_bounds(&self, cx: &mut App, store: &Store) {
        let mut config = cx.try_global::<AppConfig>().cloned().unwrap_or_default();

        for (id, (handle, hwnd, _owner)) in &self.widget_windows {
            // 使用 GPUI 内部方法获取逻辑位置（DIPs），确保与 spawn_window 时的 px(x) 保持一致
            // 这会自动处理 DPI 缩放问题
            if let Ok(bounds) = handle.update(cx, |_, window, _| window.bounds()) {
                let mut x: f32 = bounds.origin.x.into();
                let mut y: f32 = bounds.origin.y.into();
                let mut width: f32 = bounds.size.width.into();
                let mut height: f32 = bounds.size.height.into();

                let scale = handle
                    .update(cx, |_, window, _| window.scale_factor())
                    .unwrap_or(1.0);

                // 优先使用物理扩展帧边界来修正由于 WS_THICKFRAME 及阴影引起的 GPUI 偏差
                let mut actual_scale = scale;
                if *hwnd != 0 {
                    unsafe {
                        use windows_sys::Win32::Foundation::RECT;
                        use windows_sys::Win32::Graphics::Dwm::{
                            DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS,
                        };
                        use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;

                        let dpi = GetDpiForWindow(*hwnd);
                        actual_scale = if dpi == 0 { scale } else { dpi as f32 / 96.0 };

                        let mut rect: RECT = std::mem::zeroed();
                        let hr = DwmGetWindowAttribute(
                            *hwnd,
                            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
                            &mut rect as *mut _ as *mut _,
                            std::mem::size_of::<RECT>() as u32,
                        );

                        if hr == 0 {
                            x = rect.left as f32 / actual_scale;
                            y = rect.top as f32 / actual_scale;
                            width = (rect.right - rect.left) as f32 / actual_scale;
                            height = (rect.bottom - rect.top) as f32 / actual_scale;
                        } else {
                            // 降级使用 GetWindowRect
                            use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect;
                            if GetWindowRect(*hwnd, &mut rect) != 0 {
                                x = rect.left as f32 / actual_scale;
                                y = rect.top as f32 / actual_scale;
                                width = (rect.right - rect.left) as f32 / actual_scale;
                                height = (rect.bottom - rect.top) as f32 / actual_scale;
                            }
                        }
                    }
                }

                let config_for_id = config.plugins.get(*id).cloned();
                let plugin_cfg = config_for_id.unwrap_or(widget_core::PluginConfig {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                    scale: 1.0,
                    phys_x: 0,
                    phys_y: 0,
                    phys_w: 0,
                    phys_h: 0,
                    always_on_top: false,
                    mouse_passthrough: false,
                    loaded: true,
                    enabled: true,
                });
                let entry = config.plugins.entry(id.to_string()).or_insert(plugin_cfg);
                entry.x = x;
                entry.y = y;
                entry.width = width;
                entry.height = height;
                entry.scale = actual_scale;

                // 同时保存物理像素坐标（用于 SetWindowPos 精确恢复）
                if *hwnd != 0 {
                    unsafe {
                        use windows_sys::Win32::Foundation::RECT;
                        use windows_sys::Win32::Graphics::Dwm::{
                            DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS,
                        };
                        let mut rect: RECT = std::mem::zeroed();
                        let hr = DwmGetWindowAttribute(
                            *hwnd,
                            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
                            &mut rect as *mut _ as *mut _,
                            std::mem::size_of::<RECT>() as u32,
                        );
                        if hr == 0 {
                            entry.phys_x = rect.left;
                            entry.phys_y = rect.top;
                            entry.phys_w = rect.right - rect.left;
                            entry.phys_h = rect.bottom - rect.top;
                        } else {
                            use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect;
                            if GetWindowRect(*hwnd, &mut rect) != 0 {
                                entry.phys_x = rect.left;
                                entry.phys_y = rect.top;
                                entry.phys_w = rect.right - rect.left;
                                entry.phys_h = rect.bottom - rect.top;
                            }
                        }
                    }
                }

                println!(
                    "[WindowManager] 保存插件 {} 逻辑: ({}, {}) {}x{} scale={} 物理: ({}, {}) {}x{}",
                    id, x, y, width, height, actual_scale,
                    entry.phys_x, entry.phys_y, entry.phys_w, entry.phys_h
                );
            }
        }

        // 写回全局状态
        cx.set_global(config.clone());
        store.save_config(&config);
    }

    /// 获取指定插件的 Win32 HWND
    ///
    /// 返回 `0` 表示插件不存在或 HWND 尚未加载。这主要用于在异步任务中安全地引用底层窗口句柄，
    /// 而无需锁定/借用整个 `WindowManager` 或 `Context`。
    #[allow(dead_code)]
    pub fn get_plugin_hwnd(&self, plugin_id: &str) -> isize {
        self.widget_windows
            .iter()
            .find(|(k, _)| **k == plugin_id)
            .map(|(_, (_, hwnd, _))| *hwnd)
            .unwrap_or(0)
    }

    /// 控制特定窗口（基于 HWND）的可见性
    ///
    /// 此方法纯通过底层 Win32 API 操作，不依赖应用生命周期机制。
    #[allow(dead_code)]
    pub fn show_plugin_window(hwnd: isize, visible: bool) {
        if hwnd == 0 {
            return;
        }
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW};
            if visible {
                ShowWindow(hwnd, SW_SHOW);
            } else {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
        println!("[WindowManager] HWND {} 可见性: {}", hwnd, visible);
    }

    /// 应用"始终置顶"设置到所有插件窗口
    #[allow(dead_code)]
    pub fn apply_always_on_top(&self, always_on_top: bool) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
        };
        for (id, (_, hwnd, _)) in &self.widget_windows {
            if *hwnd == 0 {
                continue;
            }
            unsafe {
                let insert_after = if always_on_top {
                    HWND_TOPMOST
                } else {
                    HWND_NOTOPMOST
                };
                SetWindowPos(*hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            }
            println!("[WindowManager] 插件 {} 置顶: {}", id, always_on_top);
        }
    }

    /// 获取主窗口的 HWND（避免在 update_global 闭包内嵌套调用）
    #[allow(dead_code)]
    pub fn get_main_hwnd(&self) -> isize {
        // 只能通过已存储的值，或者在外部通过 window.update 读取
        // 这里返回 0 占位，主窗口 HWND 通过 toggle_main_window_win32 外部传入
        0
    }

    /// 通过 Win32 API 切换主窗口可见性，不持有 cx borrow
    /// 返回：(next_visible, hwnd)
    #[allow(dead_code)]
    pub fn compute_next_visible(&mut self) -> bool {
        !self.is_visible
    }

    /// 更新 is_visible 字段
    #[allow(dead_code)]
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    /// 切换主窗口显示/隐藏
    ///
    /// 此方法是纯粹基于系统底层 Win32 API (`ShowWindow`, `IsIconic` 等) 的实现，
    /// 能有效规避在部分操作闭包中获取框架级别 Window 可变引用造成的借用冲突。
    /// （前提：必须已通过某种方式正确设置了 `self.main_hwnd`）
    pub fn toggle_main_window_win32(&mut self) -> bool {
        let hwnd = self.main_hwnd;
        if hwnd == 0 {
            return self.is_visible;
        }
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE,
                SW_SHOW,
            };
            let is_win_visible = IsWindowVisible(hwnd) != 0;
            let is_minimized = IsIconic(hwnd) != 0;
            let next_visible = !is_win_visible || is_minimized;
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
    #[allow(dead_code)]
    pub fn toggle_main_window(&mut self, _cx: &mut App) {
        self.toggle_main_window_win32();
    }

    /// 将窗口附加到桌面（Progman），防止 Win + D 时被最小化
    /// 返回创建的隐藏 Owner 窗口 HWND（0 表示失败）
    pub fn attach_to_desktop(hwnd: isize) -> isize {
        if hwnd == 0 {
            return 0;
        }
        unsafe {
            // Create a dummy hidden owner window for EACH widget to prevent Win+D while avoiding Z-order grouping
            let class_name: [u16; 7] = [
                'S' as u16, 'T' as u16, 'A' as u16, 'T' as u16, 'I' as u16, 'C' as u16, 0,
            ];
            let owner = windows_sys::Win32::UI::WindowsAndMessaging::CreateWindowExW(
                windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                std::ptr::null(),
                windows_sys::Win32::UI::WindowsAndMessaging::WS_POPUP,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                std::ptr::null(),
            );
            if owner != 0 {
                SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, owner);
                println!(
                    "[WindowManager] 已将 HWND {} 附加到独立隐藏 Owner: {}",
                    hwnd, owner
                );
            }
            owner
        }
    }
}
