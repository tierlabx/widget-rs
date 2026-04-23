use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub always_on_top: bool,
    pub mouse_passthrough: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            always_on_top: true,
            mouse_passthrough: false,
        }
    }
}

pub struct Store {
    config_path: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("com", "WidgetRS", "WidgetRS")
            .expect("Could not find project directory");
        let config_dir = proj_dirs.config_dir();
        
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).unwrap();
        }

        Self {
            config_path: config_dir.join("config.json"),
        }
    }

    pub fn load_config(&self) -> AppConfig {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        }
    }

    #[allow(dead_code)]
    pub fn save_config(&self, config: &AppConfig) {
        let content = serde_json::to_string_pretty(config).unwrap();
        fs::write(&self.config_path, content).unwrap();
    }
}
