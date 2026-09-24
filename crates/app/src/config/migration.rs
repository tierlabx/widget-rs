use std::fs;
use std::path::Path;
use widget_core::AppConfig;

/// 从旧版 `config.json` 迁移配置到 SQLite
pub fn try_migrate_from_legacy_json(
    db_path: &Path,
    conn: &mut rusqlite::Connection,
    save_fn: impl FnOnce(&mut rusqlite::Connection, &AppConfig) -> rusqlite::Result<()>,
) -> Option<AppConfig> {
    let json_path = db_path.with_file_name("config.json");
    if !json_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&json_path).ok()?;
    let mut old_config: AppConfig = serde_json::from_str(&content).ok()?;

    // 规范化旧版本键名（如 todo -> todo_widget）
    let mut normalized_plugins = std::collections::HashMap::new();
    for (id, cfg) in old_config.plugins {
        let norm_id = match id.as_str() {
            "todo" => "todo_widget".to_string(),
            "sticky" => "sticky_widget".to_string(),
            "stretchly" => "stretchly_widget".to_string(),
            "fences" => "fences_widget".to_string(),
            _ => id,
        };
        normalized_plugins.insert(norm_id, cfg);
    }
    old_config.plugins = normalized_plugins;

    let mut normalized_data = std::collections::HashMap::new();
    for (id, val) in old_config.plugin_data {
        let norm_id = match id.as_str() {
            "todo" => "todo_widget".to_string(),
            "sticky" => "sticky_widget".to_string(),
            "stretchly" => "stretchly_widget".to_string(),
            "fences" => "fences_widget".to_string(),
            _ => id,
        };
        normalized_data.insert(norm_id, val);
    }
    old_config.plugin_data = normalized_data;

    println!(
        "[Store] 检测到旧版本配置文件 {:?}，正在自动迁移至 SQLite 数据库...",
        json_path
    );

    if let Err(e) = save_fn(conn, &old_config) {
        eprintln!("[Store] 迁移旧配置至 SQLite 失败: {}", e);
        return None;
    }

    println!("[Store] 旧配置已成功迁移至 SQLite 数据库");
    Some(old_config)
}
