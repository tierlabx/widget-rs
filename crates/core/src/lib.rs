use gpui::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cell::RefCell;

/// 单个插件的位置与大小配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginConfig {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// 应用全局配置（可被序列化存储）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub always_on_top: bool,
    pub mouse_passthrough: bool,
    /// 各插件位置，键为插件 ID，例如 "sticky_widget"
    pub plugins: HashMap<String, PluginConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            always_on_top: false,
            mouse_passthrough: false,
            plugins: HashMap::new(),
        }
    }
}

impl Global for AppConfig {}

/// UI 运行时状态（不持久化）
pub struct UIState {
    pub is_visible: bool,
    pub is_edit_mode: bool,
    /// 每个插件的可见状态，键为插件 ID
    pub plugin_visibility: HashMap<String, bool>,
}

impl UIState {
    /// 获取插件的可见状态（默认为 true）
    pub fn is_plugin_visible(&self, plugin_id: &str) -> bool {
        *self.plugin_visibility.get(plugin_id).unwrap_or(&true)
    }
}

impl Global for UIState {}

/// 插件 trait
pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle;
}

// ─── 线程本地 HWND 存储 ───────────────────────────────────────────────────────
// 所有操作均在主线程执行，无需跨线程同步
// widget-ui 的 on_click 可直接读取 HWND 并调用 Win32 API，完全绕开 GPUI RefCell

thread_local! {
    static PLUGIN_HWNDS: RefCell<HashMap<String, isize>> = RefCell::new(HashMap::new());
    static ALL_PLUGIN_HWND_LIST: RefCell<Vec<isize>> = RefCell::new(Vec::new());
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
