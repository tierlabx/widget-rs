use widget_core::AppConfig;
use std::fs;
use std::path::PathBuf;

pub struct Store {
    config_path: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        // 将配置存放在可执行文件所在目录
        let mut config_dir = std::env::current_exe()
            .expect("无法获取可执行文件路径");
        config_dir.pop(); // 退到父目录

        Self {
            config_path: config_dir.join("config.json"),
        }
    }

    /// 加载配置，不存在则返回默认值
    pub fn load_config(&self) -> AppConfig {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        }
    }

    /// 将配置写入磁盘
    pub fn save_config(&self, config: &AppConfig) {
        match serde_json::to_string_pretty(config) {
            Ok(content) => {
                if let Err(e) = fs::write(&self.config_path, content) {
                    eprintln!("[Store] 保存配置失败: {}", e);
                } else {
                    println!("[Store] 配置已保存到 {:?}", self.config_path);
                }
            }
            Err(e) => eprintln!("[Store] 序列化配置失败: {}", e),
        }
    }
}
