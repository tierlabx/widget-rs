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

/// 获取并确保应用高清图标文件存在（返回绝对路径）
pub fn get_app_icon_path() -> PathBuf {
    let icon_path = get_data_dir().join("app_icon.png");
    if !icon_path.exists() {
        const ICON_BYTES: &[u8] = include_bytes!("../../../assets/logos/icon.png");
        let _ = fs::write(&icon_path, ICON_BYTES);
    }
    icon_path
}

/// 获取外部扩展小部件的存放目录，并确保目录存在
pub fn get_extensions_dir() -> PathBuf {
    let ext_dir = get_data_dir().join("extensions");
    if !ext_dir.exists() {
        let _ = fs::create_dir_all(&ext_dir);
    }
    ext_dir
}

/// 获取所有可能存放外部扩展小部件的根目录列表（按优先级排序并去重）
/// 1. 用户应用数据目录：%APPDATA%/tierlabx/widget-rs/extensions
/// 2. 可执行文件同级目录：<exe_dir>/extensions
/// 3. 可执行文件资源目录：<exe_dir>/resources/extensions
/// 4. 当前工作目录（开发环境 cargo run）：./extensions
pub fn get_all_extension_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut visited = std::collections::HashSet::new();

    let mut push_dir = |p: PathBuf| {
        if p.exists() && p.is_dir() {
            if let Ok(canonical) = p.canonicalize() {
                if visited.insert(canonical) {
                    dirs.push(p);
                }
            } else if visited.insert(p.clone()) {
                dirs.push(p);
            }
        }
    };

    // 1. 用户 AppData 目录
    push_dir(get_extensions_dir());

    // 2. 可执行文件同级及资源目录（安装包环境）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            push_dir(exe_dir.join("extensions"));
            push_dir(exe_dir.join("resources").join("extensions"));
        }
    }

    // 3. 当前工作目录（开发环境 cargo run）
    if let Ok(cwd) = std::env::current_dir() {
        push_dir(cwd.join("extensions"));
    }
    push_dir(PathBuf::from("extensions"));

    dirs
}
