use crate::config::store::Store;
use widget_core::AppConfig;

pub fn sync_auto_start_with_registry(config: &mut AppConfig, store: &Store) {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_str) = exe_path.to_str() {
            let exe_path_quoted = format!("\"{}\"", exe_str);
            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
            if let Ok(run_key) = hkcu.open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                winreg::enums::KEY_ALL_ACCESS,
            ) {
                let current_val: Result<String, _> = run_key.get_value("WidgetRS");
                let mut is_enabled = false;

                let _ = run_key.delete_value("Widget RS");

                if let Ok(val) = current_val {
                    if val == exe_path_quoted {
                        is_enabled = true;
                    } else if val.contains(exe_str) {
                        let _ = run_key.set_value("WidgetRS", &exe_path_quoted);
                        is_enabled = true;
                    }
                }

                if config.auto_start != is_enabled {
                    config.auto_start = is_enabled;
                    store.save_config(&config);
                    println!(
                        "[main] 开机自启动状态与系统注册表不一致，已同步配置 auto_start = {}",
                        is_enabled
                    );
                }
            }
        }
    }
}
