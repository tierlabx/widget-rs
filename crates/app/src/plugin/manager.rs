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

    /// 批量注册外部扩展插件（自动跳过已存在的插件）
    pub fn register_external_plugins(&mut self, ext_plugins: Vec<Arc<dyn Plugin>>) {
        for ext in ext_plugins {
            let id = ext.id();
            if !self.plugins.iter().any(|p| p.id() == id) {
                self.plugins.push(ext);
            }
        }
    }

    /// 重载外部扩展插件：保留内置插件，重新填入最新的外部扩展
    pub fn reload_external_plugins(&mut self, new_exts: Vec<Arc<dyn Plugin>>) {
        self.plugins.retain(|p| !p.is_external());
        self.plugins.extend(new_exts);
    }

    /// 获取当前所有已注册的插件列表
    pub fn get_plugins(&self) -> &[Arc<dyn Plugin>] {
        &self.plugins
    }

    /// 提取全局小组件元数据列表
    pub fn build_metadata_list(&self) -> Vec<widget_core::PluginMetadata> {
        self.plugins
            .iter()
            .map(|p| widget_core::PluginMetadata {
                id: p.id().to_string().into(),
                name: p.name().to_string().into(),
                description: p.description().to_string().into(),
                icon: p.icon(),
                version: p.version().to_string().into(),
                author: p.author().to_string().into(),
                estimated_memory: p.estimated_memory(),
                has_settings: p.has_settings(),
                is_external: p.is_external(),
            })
            .collect()
    }
}
