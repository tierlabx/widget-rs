use std::path::{Path, PathBuf};
use std::sync::Arc;
use widget_core::js_plugin::JsPlugin;
use widget_core::Plugin;

/// 扫描并发现所有外部 JS 扩展插件
pub fn scan_and_load_extensions() -> Vec<Arc<dyn Plugin>> {
    // 启动时尝试同步随安装包分发的内置扩展到用户数据目录
    sync_builtin_extensions();

    let mut plugins: Vec<Arc<dyn Plugin>> = Vec::new();
    let mut scanned_paths: Vec<PathBuf> = Vec::new();
    let mut loaded_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 扫描所有已知的扩展根目录（按优先级：AppData > exe 同级 > resources > 本地开发目录）
    for ext_dir in widget_core::get_all_extension_dirs() {
        scan_directory(&ext_dir, &mut plugins, &mut scanned_paths, &mut loaded_ids);
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
    loaded_ids: &mut std::collections::HashSet<String>,
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
                        let id = js_plugin.id().to_string();
                        if loaded_ids.contains(&id) {
                            continue;
                        }
                        loaded_ids.insert(id);

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

/// 首次启动或更新时，若用户数据目录缺少默认扩展，从安装或资源目录同步
fn sync_builtin_extensions() {
    let app_data_ext = widget_core::get_extensions_dir();

    // 收集所有可能的内置扩展分发源目录
    let mut candidate_sources = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidate_sources.push(exe_dir.join("extensions"));
            candidate_sources.push(exe_dir.join("resources").join("extensions"));
        }
    }
    candidate_sources.push(PathBuf::from("extensions"));

    for src_dir in candidate_sources {
        if !src_dir.exists() || !src_dir.is_dir() {
            continue;
        }
        // 如果源目录和目标目录是同一个目录，跳过
        if let (Ok(src_c), Ok(dst_c)) = (src_dir.canonicalize(), app_data_ext.canonicalize()) {
            if src_c == dst_c {
                continue;
            }
        }

        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let src_item = entry.path();
                if src_item.is_dir() {
                    let name = entry.file_name();
                    let dst_item = app_data_ext.join(&name);
                    // 仅当用户扩展目录下不存在该小组件时才自动同步初始模板
                    if !dst_item.exists() {
                        let _ = copy_dir_all(&src_item, &dst_item);
                    }
                }
            }
        }
    }
}

/// 递归复制目录
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_child = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_child)?;
        } else {
            std::fs::copy(entry.path(), dst_child)?;
        }
    }
    Ok(())
}
