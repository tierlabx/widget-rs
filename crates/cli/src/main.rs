use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use toml_edit::{value, DocumentMut};

#[derive(Parser)]
#[command(name = "widget-cli")]
#[command(about = "Widget-rs Plugin Manager CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 插件管理
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
}

#[derive(Subcommand)]
enum PluginAction {
    /// 安装一个新插件
    Add {
        /// 插件名称 (如 custom_plugin)
        name: String,
        /// 本地路径 (如 ../../plugins/custom)
        #[arg(long)]
        path: Option<String>,
    },
    /// 移除一个已安装的插件
    Remove {
        /// 插件名称
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let workspace_root = find_workspace_root();
    let app_cargo_toml = workspace_root.join("crates/app/Cargo.toml");
    let plugin_registry_rs = workspace_root.join("crates/app/src/plugin_registry.rs");

    match &cli.command {
        Commands::Plugin { action } => match action {
            PluginAction::Add { name, path } => {
                println!("📦 正在安装插件 '{}'...", name);

                // 1. 修改 Cargo.toml
                let mut doc = read_toml(&app_cargo_toml);
                if doc["dependencies"].get(name).is_some() {
                    println!("⚠️  插件 '{}' 已存在于 Cargo.toml 中。", name);
                } else {
                    if let Some(p) = path {
                        let mut table = toml_edit::InlineTable::new();
                        table.insert("path", p.into());
                        doc["dependencies"][name] = value(table);
                    } else {
                        // 默认尝试作为一个普通的 crates.io 依赖
                        doc["dependencies"][name] = value("*");
                    }
                    write_toml(&app_cargo_toml, &doc);
                    println!("✅ 成功将 '{}' 注入 Cargo.toml", name);
                }

                // 2. 修改 plugin_registry.rs
                let mut registry_code =
                    fs::read_to_string(&plugin_registry_rs).expect("无法读取 plugin_registry.rs");
                let inject_line = format!("    pm.register({}::create_plugin());\n", name);

                if registry_code.contains(&inject_line) {
                    println!("⚠️  插件 '{}' 已在 plugin_registry.rs 中注册。", name);
                } else if let Some(pos) = registry_code.find("// [WIDGET_CLI_INJECT_PLUGINS_END]") {
                    registry_code.insert_str(pos, &inject_line);
                    fs::write(&plugin_registry_rs, registry_code)
                        .expect("无法写入 plugin_registry.rs");
                    println!("✅ 成功在 plugin_registry.rs 注册 '{}'", name);
                } else {
                    eprintln!("❌ 错误：在 plugin_registry.rs 中未找到注入标记 [WIDGET_CLI_INJECT_PLUGINS_END]");
                }
            }
            PluginAction::Remove { name } => {
                println!("🗑️ 正在移除插件 '{}'...", name);

                // 1. 修改 Cargo.toml
                let mut doc = read_toml(&app_cargo_toml);
                if doc["dependencies"].get(name).is_some() {
                    doc["dependencies"].as_table_mut().unwrap().remove(name);
                    write_toml(&app_cargo_toml, &doc);
                    println!("✅ 成功从 Cargo.toml 移除 '{}'", name);
                } else {
                    println!("⚠️  在 Cargo.toml 中未找到插件 '{}'", name);
                }

                // 2. 修改 plugin_registry.rs
                let mut registry_code =
                    fs::read_to_string(&plugin_registry_rs).expect("无法读取 plugin_registry.rs");
                let inject_line = format!("    pm.register({}::create_plugin());\n", name);
                if registry_code.contains(&inject_line) {
                    registry_code = registry_code.replace(&inject_line, "");
                    fs::write(&plugin_registry_rs, registry_code)
                        .expect("无法写入 plugin_registry.rs");
                    println!("✅ 成功从 plugin_registry.rs 取消注册 '{}'", name);
                } else {
                    println!(
                        "⚠️  在 plugin_registry.rs 中未找到插件 '{}' 的注册代码",
                        name
                    );
                }
            }
        },
    }
}

// 向上寻找包含 Cargo.toml (且有 workspace 字段) 的根目录
fn find_workspace_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap();
    loop {
        let manifest = current.join("Cargo.toml");
        if manifest.exists() {
            let content = fs::read_to_string(&manifest).unwrap();
            if content.contains("[workspace]") {
                return current;
            }
        }
        if !current.pop() {
            panic!("未找到工作区根目录");
        }
    }
}

fn read_toml(path: &PathBuf) -> DocumentMut {
    let content = fs::read_to_string(path).unwrap();
    content.parse::<DocumentMut>().expect("无效的 TOML 格式")
}

fn write_toml(path: &PathBuf, doc: &DocumentMut) {
    fs::write(path, doc.to_string()).unwrap();
}
