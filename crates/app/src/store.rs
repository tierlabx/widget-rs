use std::fs;
use std::path::PathBuf;
use widget_core::AppConfig;

/// 存储管理器
///
/// 负责应用程序配置的持久化保存与加载，支持本地文件系统操作。
pub struct Store {
    /// 配置文件存储路径（通常与可执行文件同目录下的 config.json）
    config_path: PathBuf,
}

impl Store {
    /// 创建一个新的 Store 实例，并初始化配置文件路径
    pub fn new() -> Self {
        let mut config_dir = std::env::current_exe().expect("无法获取可执行文件路径");
        config_dir.pop();

        Self {
            config_path: config_dir.join("config.json"),
        }
    }

    /// 加载配置，如果文件不存在或读取失败，则返回默认配置
    pub fn load_config(&self) -> AppConfig {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        }
    }

    /// 原子写入配置：先写 .tmp 再 rename，防止写一半崩溃损坏文件
    pub fn save_config(&self, config: &AppConfig) {
        let json = match serde_json::to_string_pretty(config) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Store] 序列化失败: {}", e);
                return;
            }
        };

        // 写入临时文件
        let tmp_path = self.config_path.with_extension("json.tmp");
        if let Err(e) = fs::write(&tmp_path, &json) {
            eprintln!("[Store] 写入临时文件失败: {}", e);
            return;
        }

        // 原子替换（同一文件系统上 rename 是原子的）
        if let Err(e) = fs::rename(&tmp_path, &self.config_path) {
            eprintln!("[Store] 原子替换失败: {}", e);
            // 尝试直接写（降级）
            let _ = fs::write(&self.config_path, &json);
        } else {
            println!("[Store] 配置已保存到 {:?}", self.config_path);
        }
    }
}
