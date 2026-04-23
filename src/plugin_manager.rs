use gpui::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle;
}

pub struct PluginManager {
    plugin_dir: PathBuf,
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut plugin_dir = std::env::current_dir().unwrap_or_default();
        plugin_dir.push("plugins");
        
        if !plugin_dir.exists() {
            let _ = fs::create_dir_all(&plugin_dir);
        }

        Self {
            plugin_dir,
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn get_plugins(&self) -> &[Arc<dyn Plugin>] {
        &self.plugins
    }

    pub fn discover_plugins(&self) -> Vec<String> {
        let mut plugins = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.plugin_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        plugins.push(name.to_string());
                    }
                }
            }
        }
        plugins
    }
}
