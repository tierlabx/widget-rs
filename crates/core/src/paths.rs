use std::fs;
use std::path::PathBuf;

/// 获取应用的项目目录（com.tierlabx.widget-rs）
pub fn get_project_dirs() -> Option<directories::ProjectDirs> {
    directories::ProjectDirs::from("com", "tierlabx", "widget-rs")
}

/// 获取崩溃与运行日志存储目录，并确保目录存在
pub fn get_log_dir() -> PathBuf {
    if let Some(proj_dirs) = get_project_dirs() {
        let log_dir = proj_dirs.data_local_dir().join("logs");
        if !log_dir.exists() {
            let _ = fs::create_dir_all(&log_dir);
        }
        log_dir
    } else {
        let mut fallback = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        fallback.pop();
        let log_dir = fallback.join("logs");
        if !log_dir.exists() {
            let _ = fs::create_dir_all(&log_dir);
        }
        log_dir
    }
}

/// 获取应用持久化数据与数据库存储目录，并确保目录存在
pub fn get_data_dir() -> PathBuf {
    if let Some(proj_dirs) = get_project_dirs() {
        let data_dir = proj_dirs.data_local_dir().to_path_buf();
        if !data_dir.exists() {
            let _ = fs::create_dir_all(&data_dir);
        }
        data_dir
    } else {
        let mut fallback = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        fallback.pop();
        fallback
    }
}
