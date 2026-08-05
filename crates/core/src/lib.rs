use gpui::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

/// 原生级别的全局编辑模式状态，供 WindowProc 直接读取
pub static NATIVE_EDIT_MODE: AtomicBool = AtomicBool::new(false);

fn default_true() -> bool {
    true
}

/// 单个插件的位置与大小配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginConfig {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub mouse_passthrough: bool,
    #[serde(default = "default_true")]
    pub loaded: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// 应用全局配置（可被序列化存储）
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub auto_start: bool,
    /// 各插件位置，键为插件 ID，例如 "sticky_widget"
    pub plugins: HashMap<String, PluginConfig>,
    /// 插件自定义数据
    #[serde(default)]
    pub plugin_data: HashMap<String, serde_json::Value>,
}

impl AppConfig {
    pub fn get_plugin_data<T: serde::de::DeserializeOwned>(&self, plugin_id: &str) -> Option<T> {
        self.plugin_data
            .get(plugin_id)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set_plugin_data<T: serde::Serialize>(&mut self, plugin_id: &str, data: &T) {
        if let Ok(value) = serde_json::to_value(data) {
            self.plugin_data.insert(plugin_id.to_string(), value);
        }
    }
}

impl Global for AppConfig {}

/// 保存回调：由 app crate 注册，插件调用以立即落盘
pub struct SaveCallback(pub std::sync::Arc<dyn Fn(&AppConfig) + Send + Sync>);
impl Global for SaveCallback {}

/// 插件位置保存回调
pub struct SaveBoundsCallback(pub std::sync::Arc<dyn Fn(&mut App)>);
impl Global for SaveBoundsCallback {}

/// 插件加载卸载切换回调
#[derive(Clone)]
pub struct TogglePluginCallback(pub std::sync::Arc<dyn Fn(&mut App, &str, bool)>);
impl Global for TogglePluginCallback {}

/// 打开插件设置窗口的回调
#[derive(Clone)]
pub struct OpenPluginSettingsCallback(pub std::sync::Arc<dyn Fn(&mut App, &str)>);
impl Global for OpenPluginSettingsCallback {}

/// 立即落盘：克隆数据后交给后台执行器执行 IO，不阻塞 GPUI 主线程
/// 在任何 GPUI 事件处理器（subscribe/listener）内都可安全调用
pub fn save_config_now(cx: &mut App) {
    // 1. 先克隆好所需数据（短暂借用，立即释放）
    let config = match cx.try_global::<AppConfig>() {
        Some(c) => c.clone(),
        None => return,
    };
    let save_fn = match cx.try_global::<SaveCallback>() {
        Some(cb) => cb.0.clone(), // 克隆 Arc，立即释放对 SaveCallback 的借用
        None => return,
    };

    // 2. 将 IO 操作派发到后台线程，完全绕开 GPUI RefCell
    cx.background_executor()
        .spawn(async move {
            save_fn(&config);
        })
        .detach();
}

pub fn save_bounds_now(cx: &mut App) {
    if let Some(cb) = cx.try_global::<SaveBoundsCallback>() {
        let cb = cb.0.clone();
        cb(cx);
    }
}

/// UI 运行时状态（不持久化）
pub struct UIState {
    pub is_visible: bool,
    pub is_edit_mode: bool,
    /// 每个插件的加载状态，键为插件 ID
    pub plugin_loaded: HashMap<String, bool>,
    /// 每个插件的启用状态，键为插件 ID
    pub plugin_enabled: HashMap<String, bool>,
}

impl UIState {
    /// 获取插件的加载状态（默认为 true）
    pub fn is_plugin_loaded(&self, plugin_id: &str) -> bool {
        *self.plugin_loaded.get(plugin_id).unwrap_or(&true)
    }

    /// 获取插件的启用状态（默认为 true）
    pub fn is_plugin_enabled(&self, plugin_id: &str) -> bool {
        *self.plugin_enabled.get(plugin_id).unwrap_or(&true)
    }
}

impl Global for UIState {}

/// 插件 trait
pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;

    fn name(&self) -> &'static str {
        self.id()
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn icon(&self) -> gpui_component::IconName {
        gpui_component::IconName::WindowMaximize
    }

    fn version(&self) -> &'static str {
        "v1.0.0"
    }

    fn author(&self) -> &'static str {
        "官方 (内置)"
    }

    fn estimated_memory(&self) -> usize {
        0
    }

    #[allow(unused_variables)]
    fn on_load(&self, cx: &mut App) {}

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle;

    #[allow(unused_variables)]
    fn on_unload(&self, cx: &mut App) {}

    #[allow(unused_variables)]
    fn build_settings_window(&self, cx: &mut App) {}
}

#[derive(Clone)]
pub struct PluginMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: gpui_component::IconName,
    pub version: &'static str,
    pub author: &'static str,
    pub estimated_memory: usize,
}

pub struct PluginList(pub Vec<PluginMetadata>);

impl Global for PluginList {}

// ─── 线程本地 HWND 存储 ───────────────────────────────────────────────────────
// 所有操作均在主线程执行，无需跨线程同步
// widget-ui 的 on_click 可直接读取 HWND 并调用 Win32 API，完全绕开 GPUI RefCell

thread_local! {
    static PLUGIN_HWNDS: RefCell<HashMap<String, isize>> = RefCell::new(HashMap::new());
    static ALL_PLUGIN_HWND_LIST: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

/// 注册插件 HWND（由 app crate 在 HWND 提取后调用）
pub fn register_plugin_hwnd(id: &str, hwnd: isize) {
    PLUGIN_HWNDS.with(|m| {
        m.borrow_mut().insert(id.to_string(), hwnd);
    });
    ALL_PLUGIN_HWND_LIST.with(|v| {
        let mut list = v.borrow_mut();
        if !list.contains(&hwnd) {
            list.push(hwnd);
        }
    });
}

/// 获取指定插件的 HWND
pub fn get_plugin_hwnd(id: &str) -> isize {
    PLUGIN_HWNDS.with(|m| *m.borrow().get(id).unwrap_or(&0))
}

/// 获取所有插件的 HWND 列表（用于批量操作）
pub fn get_all_plugin_hwnds() -> Vec<isize> {
    ALL_PLUGIN_HWND_LIST.with(|v| v.borrow().clone())
}

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
