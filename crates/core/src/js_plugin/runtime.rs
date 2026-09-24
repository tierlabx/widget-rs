use anyhow::Result;
use gpui::{App, Global};
use std::rc::Rc;

/// 全局 gpui-shell 运行时包装容器
#[derive(Clone)]
struct GlobalShellRuntime(Rc<gpui_shell::ShellRuntime>);

impl Global for GlobalShellRuntime {}

/// 初始化 gpui-shell 脚本引擎并注册到全局应用上下文
pub fn init_shell(cx: &mut App) -> Result<Rc<gpui_shell::ShellRuntime>> {
    // 1. 初始化 gpui-shell 基础样式与类型反射系统
    gpui_shell::init(cx);

    // 2. 授权网络访问白名单（和风天气、IP自动定位、开放天气等接口域名）
    let mut network_hosts = vec![
        "devapi.qweather.com".to_string(),
        "api.qweather.com".to_string(),
        "geoapi.qweather.com".to_string(),
        "m67fbv5m3d.re.qweatherapi.com".to_string(),
        "ip-api.com".to_string(),
        "api.ip.sb".to_string(),
        "ipapi.co".to_string(),
        "qifu-api.baidubce.com".to_string(),
        "ipwho.is".to_string(),
        "api.open-meteo.com".to_string(),
        "wttr.in".to_string(),
    ];
    scan_custom_plugin_hosts(&mut network_hosts);

    let capabilities = gpui_shell::Capabilities::new().network_hosts(network_hosts);
    gpui_shell::set_capabilities(capabilities);

    // 3. 构造默认脚本运行时实例
    let runtime = gpui_shell::ShellRuntime::new(cx)?;
    cx.set_global(GlobalShellRuntime(runtime.clone()));

    // 3. 导出通用宿主模块 widget-rs 供 JavaScript 扩展按需导入
    let host_module = gpui_shell::HostModule::new("widget-rs")
        .declarations(
            r#"
            export function version(): string;
            export function app_name(): string;
            export function read_config(plugin_id: string): string;
            "#,
        )
        .function("version", |_| {
            Ok(gpui_shell::HostValue::from(env!("CARGO_PKG_VERSION")))
        })
        .function("app_name", |_| Ok(gpui_shell::HostValue::from("widget-rs")))
        .function("read_config", |args| {
            let plugin_id = match args.get(0) {
                Some(gpui_shell::HostValue::Str(s)) => s.as_str(),
                _ => "",
            };
            Ok(gpui_shell::HostValue::from(read_plugin_config(plugin_id)))
        });

    let _ = gpui_shell::export_module(host_module);

    Ok(runtime)
}

/// 从全局 App 上下文中获取已初始化的 gpui-shell 运行时实例
pub fn get_shell_runtime(cx: &App) -> Option<Rc<gpui_shell::ShellRuntime>> {
    cx.try_global::<GlobalShellRuntime>().map(|g| g.0.clone())
}

/// 确保 gpui-shell 脚本引擎已按需懒加载初始化
pub fn ensure_shell_runtime(cx: &mut App) -> Result<Rc<gpui_shell::ShellRuntime>> {
    if let Some(runtime) = get_shell_runtime(cx) {
        return Ok(runtime);
    }
    init_shell(cx)
}

/// 读取插件的配置数据（优先读取本地私有 config.json，其次读取 manifest.json 中的 settings 字段）
fn read_plugin_config(plugin_id: &str) -> String {
    let candidate_dirs: Vec<std::path::PathBuf> = crate::get_all_extension_dirs()
        .into_iter()
        .map(|d| d.join(plugin_id))
        .collect();

    // 1. 优先读取私有 config.json（已加入 .gitignore，保护用户个人私密配置）
    for dir in &candidate_dirs {
        let config_path = dir.join("config.json");
        if config_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                return content;
            }
        }
    }

    // 2. 其次回退读取 manifest.json 中的 settings 字段
    for dir in &candidate_dirs {
        let manifest_path = dir.join("manifest.json");
        if manifest_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(settings) = val.get("settings") {
                        if !settings.is_null() {
                            return settings.to_string();
                        }
                    }
                }
            }
        }
    }

    "{}".to_string()
}

/// 自动扫描本地及用户扩展目录，提取文件中的自定义和风天气等主机名并加入白名单
fn scan_custom_plugin_hosts(hosts: &mut Vec<String>) {
    let candidate_dirs = crate::get_all_extension_dirs();

    for base_dir in candidate_dirs {
        if !base_dir.exists() || !base_dir.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    for filename in ["main.js", "config.json", "manifest.json"] {
                        let file_path = path.join(filename);
                        if let Ok(content) = std::fs::read_to_string(&file_path) {
                            for word in content.split(|c: char| {
                                c.is_whitespace() || c == '"' || c == '\'' || c == '`'
                            }) {
                                if word.contains(".qweatherapi.com")
                                    || word.contains(".qweather.com")
                                {
                                    let host = word
                                        .trim_start_matches("https://")
                                        .trim_start_matches("http://")
                                        .split('/')
                                        .next()
                                        .unwrap_or("")
                                        .to_ascii_lowercase();
                                    if !host.is_empty() && !hosts.contains(&host) {
                                        hosts.push(host);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dump_typings() {
        let mut decls = gpui_shell::type_declarations(&Default::default());
        decls.push_str("\n\ndeclare module \"widget-rs\" {\n    export function version(): string;\n    export function app_name(): string;\n    export function read_config(plugin_id: string): string;\n}\n");
        let _ = std::fs::write("../../extensions/clock/gpui-kit.d.ts", &decls);
        let _ = std::fs::write("../../extensions/gpui-kit.d.ts", &decls);
    }
}
