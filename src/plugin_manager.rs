use std::fs;
use std::path::PathBuf;

pub struct PluginManager {
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut plugin_dir = std::env::current_dir().unwrap_or_default();
        plugin_dir.push("plugins");
        
        if !plugin_dir.exists() {
            let _ = fs::create_dir_all(&plugin_dir);
        }

        Self { plugin_dir }
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

    // Dynamic UI loading will be implemented here using slint::interpreter
}
