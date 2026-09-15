use std::path::{Path, PathBuf};
use std::sync::Arc;
use widget_core::js_plugin::JsPlugin;
use widget_core::Plugin;

/// 扫描并发现所有外部 JS 扩展插件
pub fn scan_and_load_extensions() -> Vec<Arc<dyn Plugin>> {
    let mut plugins: Vec<Arc<dyn Plugin>> = Vec::new();
    let mut scanned_paths: Vec<PathBuf> = Vec::new();

    // 1. 用户应用数据目录: %APPDATA%/tierlabx/widget-rs/extensions
    let app_data_ext_dir = widget_core::get_extensions_dir();
    scan_directory(&app_data_ext_dir, &mut plugins, &mut scanned_paths);

    // 2. 本地开发环境目录: ./extensions
    let local_ext_dir = PathBuf::from("extensions");
    if local_ext_dir.exists() && local_ext_dir.is_dir() {
        scan_directory(&local_ext_dir, &mut plugins, &mut scanned_paths);
    }

    println!(
        "[ExtensionLoader] 扫描完成，成功装载 {} 个外部 JS 扩展小组件",
        plugins.len()
    );
    plugins
}

/// 扫描特定目录下的所有子目录小组件包
fn scan_directory(
    base_dir: &Path,
    plugins: &mut Vec<Arc<dyn Plugin>>,
    scanned_paths: &mut Vec<PathBuf>,
) {
    if !base_dir.exists() || !base_dir.is_dir() {
        return;
    }

    let entries = match std::fs::read_dir(base_dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("[ExtensionLoader] 读取目录 {:?} 失败: {}", base_dir, err);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // 如果该目录有 manifest.json、widget.json 或 gpui-shell.json，尝试加载
            let has_manifest = path.join("manifest.json").exists()
                || path.join("widget.json").exists()
                || path.join("gpui-shell.json").exists();
            if has_manifest {
                // 避免重复扫描相同绝对路径
                if let Ok(canonical) = path.canonicalize() {
                    if scanned_paths.contains(&canonical) {
                        continue;
                    }
                    scanned_paths.push(canonical);
                }

                match JsPlugin::load_from_dir(&path) {
                    Ok(js_plugin) => {
                        println!(
                            "[ExtensionLoader] 成功加载外部 JS 扩展: [{}] {} (版本 {})",
                            js_plugin.id(),
                            js_plugin.name(),
                            js_plugin.version()
                        );
                        plugins.push(Arc::new(js_plugin));
                    }
                    Err(err) => {
                        eprintln!("[ExtensionLoader] 加载扩展目录 {:?} 失败: {}", path, err);
                    }
                }
            }
        }
    }
}
