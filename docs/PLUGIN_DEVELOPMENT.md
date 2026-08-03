# Widget-RS 插件开发指南

Widget-RS 拥有一个高性能且易于使用的源码级插件系统。通过本文档，你可以快速学会如何从零开始开发一个自己的桌面小部件插件，并使用 `widget-cli` 将其集成。

## 1. 插件系统架构简介

Widget-RS 的底层架构将 **核心框架 (`widget-core`)** 与 **业务插件 (`plugins/*`)** 完全解耦。
- **渲染核心**：基于 GPUI，提供极致的原生渲染性能和极小的内存占用。
- **动态注册**：通过独立的注册表文件 (`plugin_registry.rs`) 与自动化 CLI 工具配合，实现插件的一键安装与卸载，开发者无需手动修改主程序。

## 2. 快速创建插件

你可以使用 Cargo 命令直接在 `plugins` 目录下新建一个插件工程：
```bash
cargo new --lib plugins/my_clock
```

### 2.1 添加依赖
在 `plugins/my_clock/Cargo.toml` 中，你需要引入 GPUI 和 Widget-RS 核心库：
```toml
[package]
name = "my_clock"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui.workspace = true
widget-core.workspace = true
```

### 2.2 实现插件 Trait
在 `plugins/my_clock/src/lib.rs` 中，定义你的小部件和插件入口：

```rust
use gpui::*;
use widget_core::{Plugin, AppConfig};

// 1. 定义小部件 UI
struct ClockWidget {
    // 你的组件状态
}

impl ClockWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for ClockWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xFFFFFF))
            .child("Hello, Widget!")
    }
}

// 2. 定义插件结构
pub struct ClockPlugin;

impl Plugin for ClockPlugin {
    fn name(&self) -> &str {
        "my_clock"
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::PopUp,
            is_resizable: false,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::new(px(100.0), px(100.0)),
                size(px(200.0), px(100.0)),
            ))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| ClockWidget::new(window, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        })
        .unwrap()
        .into()
    }
}

// 3. (关键!) 暴露标准的插件入口函数，供 CLI 自动化调用
pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(ClockPlugin)
}
```

## 3. 使用 CLI 安装插件

开发完成后，无需手动修改 `main.rs`。在项目根目录下运行 `widget-cli`：

```bash
cargo run -p widget-cli -- plugin add my_clock --path plugins/my_clock
```
该命令会自动：
1. 修改 `crates/app/Cargo.toml` 注入你的依赖。
2. 扫描 `crates/app/src/plugin_registry.rs` 并插入 `my_clock::create_plugin()`。

安装完成后，直接运行主程序即可看到你的小部件！
```bash
cargo run
```

## 4. 插件数据持久化

如果你希望保存你的插件数据，请直接使用 `widget_core::AppConfig` 提供的泛型接口：
```rust
// 写入数据
cx.update_global::<AppConfig, _>(|cfg, _| {
    cfg.set_plugin_data("my_clock", &my_data); // my_data 必须实现 serde::Serialize
});
widget_core::save_config_now(cx); // 立即落盘

// 读取数据
let saved_data: MyData = cx
    .try_global::<AppConfig>()
    .and_then(|cfg| cfg.get_plugin_data::<MyData>("my_clock"))
    .unwrap_or_default();
```

## 5. 卸载插件
如果你不再需要这个插件，同样通过 CLI 卸载：
```bash
cargo run -p widget-cli -- plugin remove my_clock
```
