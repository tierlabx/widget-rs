use std::fs;
use std::path::PathBuf;
use widget_core::{AppConfig, PluginConfig};

use super::migration;

/// SQLite 数据库存储管理器
///
/// 负责应用程序配置的持久化保存与加载，基于 SQLite 关系型存储。
pub struct Store {
    /// SQLite 数据库文件路径（通常为 `%APPDATA%\tierlabx\widget-rs\config.db`）
    db_path: PathBuf,
}

impl Store {
    /// 创建一个新的 Store 实例，并初始化数据库路径与数据表
    pub fn new() -> Self {
        let db_path = if let Some(proj_dirs) =
            directories::ProjectDirs::from("com", "tierlabx", "widget-rs")
        {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir);
            }
            config_dir.join("config.db")
        } else {
            // 后备方案：如果获取不到系统的 AppData 目录，则回退到可执行文件同级目录
            let mut fallback = std::env::current_exe().expect("无法获取可执行文件路径");
            fallback.pop();
            fallback.join("config.db")
        };

        let store = Self { db_path };
        if let Err(e) = store.init_db() {
            eprintln!("[Store] 初始化数据库失败: {}", e);
        }
        store
    }

    /// 使用指定数据库路径创建 Store 实例（主要用于测试或自定义路径）
    #[allow(dead_code)]
    pub fn with_path(db_path: PathBuf) -> Self {
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }
        let store = Self { db_path };
        if let Err(e) = store.init_db() {
            eprintln!("[Store] 初始化数据库失败: {}", e);
        }
        store
    }

    /// 获取底层数据库连接并初始化性能参数
    fn open_connection(&self) -> rusqlite::Result<rusqlite::Connection> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;
        Ok(conn)
    }

    /// 初始化数据表架构
    fn init_db(&self) -> rusqlite::Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS plugin_configs (
                id TEXT PRIMARY KEY,
                x REAL NOT NULL,
                y REAL NOT NULL,
                width REAL NOT NULL,
                height REAL NOT NULL,
                scale REAL NOT NULL DEFAULT 1.0,
                phys_x INTEGER NOT NULL DEFAULT 0,
                phys_y INTEGER NOT NULL DEFAULT 0,
                phys_w INTEGER NOT NULL DEFAULT 0,
                phys_h INTEGER NOT NULL DEFAULT 0,
                always_on_top INTEGER NOT NULL DEFAULT 0,
                mouse_passthrough INTEGER NOT NULL DEFAULT 0,
                pinned_to_desktop INTEGER NOT NULL DEFAULT 0,
                loaded INTEGER NOT NULL DEFAULT 1,
                enabled INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS plugin_data (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    /// 检查数据库是否为空（无任何设置和插件数据）
    fn is_db_empty(&self, conn: &rusqlite::Connection) -> bool {
        let count_settings: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_settings", [], |r| r.get(0))
            .unwrap_or(0);
        let count_plugins: i64 = conn
            .query_row("SELECT COUNT(*) FROM plugin_configs", [], |r| r.get(0))
            .unwrap_or(0);
        let count_data: i64 = conn
            .query_row("SELECT COUNT(*) FROM plugin_data", [], |r| r.get(0))
            .unwrap_or(0);

        count_settings == 0 && count_plugins == 0 && count_data == 0
    }

    /// 加载配置，若数据库为空且存在历史 JSON 则自动迁移，若读取失败则返回默认配置
    pub fn load_config(&self) -> AppConfig {
        let mut conn = match self.open_connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[Store] 打开数据库失败: {}", e);
                return AppConfig::default();
            }
        };

        // 如果数据库为空，尝试从历史 config.json 迁移
        if self.is_db_empty(&conn) {
            if let Some(migrated) =
                migration::try_migrate_from_legacy_json(&self.db_path, &mut conn, |c, cfg| {
                    Self::save_config_internal(c, cfg)
                })
            {
                return migrated;
            }
        }

        let mut config = AppConfig::default();

        // 1. 读取 app_settings
        if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM app_settings") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }) {
                for item in rows.flatten() {
                    match item.0.as_str() {
                        "auto_start" => config.auto_start = item.1.parse().unwrap_or(false),
                        "silent_start" => config.silent_start = item.1.parse().unwrap_or(false),
                        "auto_check_update" => {
                            config.auto_check_update = item.1.parse().unwrap_or(true)
                        }
                        _ => {}
                    }
                }
            }
        }

        // 2. 读取 plugin_configs
        if let Ok(mut stmt) = conn.prepare(
            "SELECT id, x, y, width, height, scale, phys_x, phys_y, phys_w, phys_h,
                    always_on_top, mouse_passthrough, pinned_to_desktop, loaded, enabled
             FROM plugin_configs",
        ) {
            let plugin_map = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let plugin_cfg = PluginConfig {
                    x: row.get(1)?,
                    y: row.get(2)?,
                    width: row.get(3)?,
                    height: row.get(4)?,
                    scale: row.get(5)?,
                    phys_x: row.get(6)?,
                    phys_y: row.get(7)?,
                    phys_w: row.get(8)?,
                    phys_h: row.get(9)?,
                    always_on_top: row.get::<_, i32>(10)? != 0,
                    mouse_passthrough: row.get::<_, i32>(11)? != 0,
                    pinned_to_desktop: row.get::<_, i32>(12)? != 0,
                    loaded: row.get::<_, i32>(13)? != 0,
                    enabled: row.get::<_, i32>(14)? != 0,
                };
                Ok((id, plugin_cfg))
            });

            if let Ok(rows) = plugin_map {
                for item in rows.flatten() {
                    config.plugins.insert(item.0, item.1);
                }
            }
        }

        // 3. 读取 plugin_data
        if let Ok(mut stmt) = conn.prepare("SELECT id, data FROM plugin_data") {
            let data_map = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let raw_json: String = row.get(1)?;
                let val: serde_json::Value =
                    serde_json::from_str(&raw_json).unwrap_or(serde_json::Value::Null);
                Ok((id, val))
            });

            if let Ok(rows) = data_map {
                for item in rows.flatten() {
                    config.plugin_data.insert(item.0, item.1);
                }
            }
        }

        config
    }

    /// 事务性保存配置到数据库内部方法
    fn save_config_internal(
        conn: &mut rusqlite::Connection,
        config: &AppConfig,
    ) -> rusqlite::Result<()> {
        let tx = conn.transaction()?;

        // 1. 保存 app_settings
        {
            let mut stmt = tx.prepare(
                "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )?;
            stmt.execute(rusqlite::params![
                "auto_start",
                config.auto_start.to_string()
            ])?;
            stmt.execute(rusqlite::params![
                "silent_start",
                config.silent_start.to_string()
            ])?;
            stmt.execute(rusqlite::params![
                "auto_check_update",
                config.auto_check_update.to_string()
            ])?;
        }

        // 2. 保存 plugin_configs
        {
            // 清理旧的记录并写入最新记录，保证与 HashMap 状态严格一致
            tx.execute("DELETE FROM plugin_configs", [])?;
            let mut stmt = tx.prepare(
                "INSERT INTO plugin_configs (
                    id, x, y, width, height, scale, phys_x, phys_y, phys_w, phys_h,
                    always_on_top, mouse_passthrough, pinned_to_desktop, loaded, enabled
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            )?;
            for (id, p) in &config.plugins {
                stmt.execute(rusqlite::params![
                    id,
                    p.x,
                    p.y,
                    p.width,
                    p.height,
                    p.scale,
                    p.phys_x,
                    p.phys_y,
                    p.phys_w,
                    p.phys_h,
                    if p.always_on_top { 1 } else { 0 },
                    if p.mouse_passthrough { 1 } else { 0 },
                    if p.pinned_to_desktop { 1 } else { 0 },
                    if p.loaded { 1 } else { 0 },
                    if p.enabled { 1 } else { 0 },
                ])?;
            }
        }

        // 3. 保存 plugin_data
        {
            tx.execute("DELETE FROM plugin_data", [])?;
            let mut stmt = tx.prepare("INSERT INTO plugin_data (id, data) VALUES (?1, ?2)")?;
            for (id, val) in &config.plugin_data {
                let json_str = serde_json::to_string(val).unwrap_or_default();
                stmt.execute(rusqlite::params![id, json_str])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// 保存配置到 SQLite 数据库
    pub fn save_config(&self, config: &AppConfig) {
        let mut conn = match self.open_connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[Store] 建立数据库连接失败: {}", e);
                return;
            }
        };

        if let Err(e) = Self::save_config_internal(&mut conn, config) {
            eprintln!("[Store] 保存配置失败: {}", e);
        } else {
            println!("[Store] 配置已保存到 SQLite 数据库 {:?}", self.db_path);
        }
    }
}
