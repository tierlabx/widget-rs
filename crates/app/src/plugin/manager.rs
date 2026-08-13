use std::sync::Arc;

pub use widget_core::Plugin;

/// 插件管理器
///
/// 负责注册和管理系统中所有小组件（Widget）插件。
#[derive(Clone)]
pub struct PluginManager {
    /// 已注册的插件列表，使用 Arc 共享以供并发/跨线程使用
    plugins: Vec<Arc<dyn Plugin>>,
}

impl gpui::Global for PluginManager {}

impl PluginManager {
    /// 创建一个新的插件管理器实例
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// 注册一个新的插件
    ///
    /// # 参数
    /// * `plugin` - 要注册的插件，需要实现 `Plugin` trait
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// 获取当前所有已注册的插件列表
    pub fn get_plugins(&self) -> &[Arc<dyn Plugin>] {
        &self.plugins
    }
}
