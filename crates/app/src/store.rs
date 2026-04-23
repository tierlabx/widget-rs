use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub always_on_top: bool,
    pub mouse_passthrough: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            always_on_top: false,
            mouse_passthrough: false,
        }
    }
}

pub struct Store {
    config_path: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let mut config_dir = std::env::current_exe()
            .expect("Could not find current executable path");
        config_dir.pop(); // Go to the parent directory

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
