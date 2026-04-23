use gpui::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

impl Global for UIState {}

/// 插件 trait
pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle;
}
