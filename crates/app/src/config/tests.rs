use super::store::Store;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use widget_core::{AppConfig, PluginConfig};

fn create_test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "widget_rs_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::create_dir_all(&dir);
    dir
}

#[test]
fn test_default_config_loading() {
    let temp_dir = create_test_dir();
    let db_path = temp_dir.join("config.db");
    let store = Store::with_path(db_path.clone());

    let config = store.load_config();
    assert!(!config.auto_start);
    assert!(config.auto_check_update);
    assert!(config.plugins.is_empty());
    assert!(config.plugin_data.is_empty());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_sqlite_store_roundtrip() {
    let temp_dir = create_test_dir();
    let db_path = temp_dir.join("config.db");
    let store = Store::with_path(db_path.clone());

    let mut initial_config = AppConfig::default();
    initial_config.auto_start = true;
    initial_config.auto_check_update = false;

    let plugin_cfg = PluginConfig {
        x: 120.5,
        y: 240.5,
        width: 300.0,
        height: 450.0,
        scale: 1.25,
        phys_x: 150,
        phys_y: 300,
        phys_w: 375,
        phys_h: 562,
        always_on_top: true,
        mouse_passthrough: false,
        pinned_to_desktop: true,
        loaded: true,
        enabled: true,
    };
    initial_config
        .plugins
        .insert("test_plugin".to_string(), plugin_cfg);
    initial_config.plugin_data.insert(
        "test_plugin".to_string(),
        json!({
            "items": ["item1", "item2"],
            "color": "#ff0000",
            "count": 42
        }),
    );

    // 1. 保存到 SQLite
    store.save_config(&initial_config);

    // 2. 重新加载并校验
    let loaded = store.load_config();
    assert!(loaded.auto_start);
    assert!(!loaded.auto_check_update);
    let p = loaded.plugins.get("test_plugin").unwrap();
    assert_eq!(p.x, 120.5);
    assert_eq!(p.y, 240.5);
    assert_eq!(p.scale, 1.25);
    assert_eq!(p.phys_x, 150);
    assert_eq!(p.phys_y, 300);
    assert_eq!(p.phys_w, 375);
    assert_eq!(p.phys_h, 562);
    assert!(p.always_on_top);
    assert!(!p.mouse_passthrough);
    assert!(p.pinned_to_desktop);
    assert!(p.loaded);
    assert!(p.enabled);

    let data = loaded.plugin_data.get("test_plugin").unwrap();
    assert_eq!(data["color"], "#ff0000");
    assert_eq!(data["count"], 42);
    assert_eq!(data["items"][0], "item1");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_plugin_config_mutation_and_deletion() {
    let temp_dir = create_test_dir();
    let db_path = temp_dir.join("config.db");
    let store = Store::with_path(db_path.clone());

    let mut config = AppConfig::default();
    config.plugins.insert(
        "plugin_a".to_string(),
        PluginConfig {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 100.0,
            scale: 1.0,
            phys_x: 10,
            phys_y: 20,
            phys_w: 100,
            phys_h: 100,
            always_on_top: false,
            mouse_passthrough: false,
            pinned_to_desktop: false,
            loaded: true,
            enabled: true,
        },
    );
    config.plugins.insert(
        "plugin_b".to_string(),
        PluginConfig {
            x: 50.0,
            y: 60.0,
            width: 200.0,
            height: 200.0,
            scale: 1.0,
            phys_x: 50,
            phys_y: 60,
            phys_w: 200,
            phys_h: 200,
            always_on_top: true,
            mouse_passthrough: false,
            pinned_to_desktop: true,
            loaded: true,
            enabled: true,
        },
    );
    store.save_config(&config);

    // 删除 plugin_a，修改 plugin_b，新增 plugin_c
    config.plugins.remove("plugin_a");
    if let Some(b) = config.plugins.get_mut("plugin_b") {
        b.x = 999.0;
        b.always_on_top = false;
    }
    config.plugins.insert(
        "plugin_c".to_string(),
        PluginConfig {
            x: 300.0,
            y: 400.0,
            width: 500.0,
            height: 500.0,
            scale: 1.5,
            phys_x: 450,
            phys_y: 600,
            phys_w: 750,
            phys_h: 750,
            always_on_top: true,
            mouse_passthrough: true,
            pinned_to_desktop: false,
            loaded: false,
            enabled: false,
        },
    );
    store.save_config(&config);

    let loaded = store.load_config();
    assert!(!loaded.plugins.contains_key("plugin_a"));
    assert_eq!(loaded.plugins.get("plugin_b").unwrap().x, 999.0);
    assert!(!loaded.plugins.get("plugin_b").unwrap().always_on_top);
    let c = loaded.plugins.get("plugin_c").unwrap();
    assert_eq!(c.x, 300.0);
    assert_eq!(c.scale, 1.5);
    assert!(c.always_on_top);
    assert!(c.mouse_passthrough);
    assert!(!c.loaded);
    assert!(!c.enabled);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_migrate_from_legacy_json() {
    let temp_dir = create_test_dir();
    let json_path = temp_dir.join("config.json");
    let db_path = temp_dir.join("config.db");

    // 写入旧格式 json
    let legacy_json = r#"{
        "auto_start": true,
        "auto_check_update": true,
        "plugins": {
            "sticky_widget": {
                "x": 100.0,
                "y": 200.0,
                "width": 300.0,
                "height": 400.0,
                "scale": 1.0,
                "phys_x": 100,
                "phys_y": 200,
                "phys_w": 300,
                "phys_h": 400,
                "always_on_top": true,
                "mouse_passthrough": false,
                "pinned_to_desktop": false,
                "loaded": true,
                "enabled": true
            }
        },
        "plugin_data": {
            "sticky_widget": {
                "text": "Hello Legacy"
            }
        }
    }"#;
    fs::write(&json_path, legacy_json).unwrap();

    // 创建 Store 并调用 load_config
    let store = Store::with_path(db_path.clone());
    let config = store.load_config();

    assert!(config.auto_start);
    assert!(config.auto_check_update);
    assert!(config.plugins.contains_key("sticky_widget"));
    assert_eq!(config.plugins["sticky_widget"].x, 100.0);
    assert_eq!(config.plugin_data["sticky_widget"]["text"], "Hello Legacy");

    // 验证数据库文件已经成功生成
    assert!(db_path.exists());

    // 再次从数据库加载，验证即使 json 被删除，数据库中依然有完整数据
    fs::remove_file(&json_path).unwrap();
    let reload = store.load_config();
    assert_eq!(reload.plugins["sticky_widget"].width, 300.0);
    assert_eq!(reload.plugin_data["sticky_widget"]["text"], "Hello Legacy");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn bench_sqlite_bulk_config_io() {
    let temp_dir = create_test_dir();
    let db_path = temp_dir.join("bench_config.db");
    let store = Store::with_path(db_path);

    let mut config = AppConfig::default();
    for i in 0..20 {
        config.plugins.insert(
            format!("plugin_{i}"),
            PluginConfig {
                x: i as f32 * 10.0,
                y: i as f32 * 10.0,
                width: 300.0,
                height: 400.0,
                scale: 1.0,
                phys_x: i as i32 * 10,
                phys_y: i as i32 * 10,
                phys_w: 300,
                phys_h: 400,
                always_on_top: true,
                mouse_passthrough: false,
                pinned_to_desktop: false,
                loaded: true,
                enabled: true,
            },
        );
        config.plugin_data.insert(
            format!("plugin_{i}"),
            json!({
                "key": format!("value_{i}"),
                "numbers": [1, 2, 3, 4, 5],
            }),
        );
    }

    let iterations = 100;
    let start_save = std::time::Instant::now();
    for _ in 0..iterations {
        store.save_config(&config);
    }
    let save_duration = start_save.elapsed();
    println!(
        "[性能测试] SQLite 连续保存 {iterations} 次多插件配置耗时: {:?}",
        save_duration
    );

    let start_load = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = store.load_config();
    }
    let load_duration = start_load.elapsed();
    println!(
        "[性能测试] SQLite 连续读取 {iterations} 次多插件配置耗时: {:?}",
        load_duration
    );

    assert!(
        save_duration.as_secs() < 5,
        "100次批量写入应在合理时间内完成"
    );
    assert!(
        load_duration.as_secs() < 2,
        "100次批量读取应在合理时间内完成"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
