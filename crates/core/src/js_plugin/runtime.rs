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

    // 2. 构造默认脚本运行时实例
    let runtime = gpui_shell::ShellRuntime::new(cx)?;
    cx.set_global(GlobalShellRuntime(runtime.clone()));

    // 3. 导出通用宿主模块 widget-rs 供 JavaScript 扩展按需导入
    let host_module = gpui_shell::HostModule::new("widget-rs")
        .declarations(
            r#"
            export function version(): string;
            export function app_name(): string;
            "#,
        )
        .function("version", |_| {
            Ok(gpui_shell::HostValue::from(env!("CARGO_PKG_VERSION")))
        })
        .function("app_name", |_| Ok(gpui_shell::HostValue::from("widget-rs")));

    let _ = gpui_shell::export_module(host_module);

    Ok(runtime)
}

/// 从全局 App 上下文中获取已初始化的 gpui-shell 运行时实例
pub fn get_shell_runtime(cx: &App) -> Option<Rc<gpui_shell::ShellRuntime>> {
    cx.try_global::<GlobalShellRuntime>().map(|g| g.0.clone())
}
