# Widget-RS 插件开发指南

Widget-RS 拥有一个高性能且易于使用的源码级插件系统。通过本文档，你可以快速学会如何从零开始开发一个自己的桌面小部件插件，并使用 `widget-cli` 将其集成。

## 1. 插件系统架构简介

Widget-RS 的底层架构将 **核心框架 (`widget-core`)** 与 **业务插件 (`plugins/*`)** 完全解耦。
- **渲染核心**：基于 GPUI，提供极致的原生渲染性能和极小的内存占用。
- **窗口容器**：`WidgetWindow<T>` 统一封装了所有小组件窗口的通用能力（编辑模式、拖拽条、窗口边框、穿透、固定等），插件开发者无需关心窗口级细节。
- **动态注册**：通过独立的注册表文件 (`plugin_registry.rs`) 与自动化 CLI 工具配合，实现插件的一键安装与卸载，开发者无需手动修改主程序。

## 2. 核心概念

### 2.1 WidgetContent trait

`WidgetContent` 是插件内容的最小接口，插件只需关注"画什么"：

```rust
pub trait WidgetContent: Render + Sized + 'static {
    /// 返回插件 ID（与 Plugin::id 一致）
    fn plugin_id(&self) -> &'static str;

    /// 编辑模式拖拽条上的标签文字
    fn drag_label(&self) -> &'static str { "拖拽移动" }

    /// 是否显示编辑模式拖拽条（默认 true）
    fn show_drag_handle(&self) -> bool { true }
}
```

### 2.2 WidgetWindow 容器

`WidgetWindow<T: WidgetContent>` 自动为你的插件提供：
- 编辑模式检测与窗口样式切换
- 编辑模式拖拽条（绿色 `#00d992`，支持原生窗口拖拽）
- 编辑模式边框高亮

你**不需要**在插件中手动处理这些逻辑。

## 3. 快速创建插件

你可以使用 Cargo 命令直接在 `plugins` 目录下新建一个插件工程：
```bash
cargo new --lib plugins/my_clock
```

### 3.1 添加依赖
在 `plugins/my_clock/Cargo.toml` 中，你需要引入 GPUI 和 Widget-RS 核心库：
```toml
[package]
name = "my_clock"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui.workspace = true
gpui-component.workspace = true
widget-core.workspace = true
```

### 3.2 实现插件

在 `plugins/my_clock/src/lib.rs` 中，定义你的小部件和插件入口：

```rust
use gpui::*;
use widget_core::Plugin;

// 1. 定义小部件 UI
struct ClockWidget {
    // 你的组件状态
}

impl ClockWidget {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

// 2. 实现 WidgetContent — 只需提供 plugin_id 和 drag_label
impl widget_core::WidgetContent for ClockWidget {
    fn plugin_id(&self) -> &'static str { "my_clock" }
    fn drag_label(&self) -> &'static str { "拖拽移动时钟" }
}

// 3. 实现 Render — 只画你的内容，不需要关心编辑模式/拖拽/边框
impl Render for ClockWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .justify_center()
            .items_center()
            .size_full()
            .bg(rgba(0x050507d9))
            .text_xl()
            .text_color(rgb(0xFFFFFF))
            .child("Hello, Widget!")
    }
}

// 4. 定义插件结构
pub struct ClockPlugin;

impl Plugin for ClockPlugin {
    fn id(&self) -> &'static str { "my_clock" }
    fn name(&self) -> &'static str { "我的时钟" }
    fn description(&self) -> &'static str { "一个简单的桌面时钟小部件" }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        // 使用辅助函数创建标准窗口选项
        let options = widget_core::default_widget_window_options(
            cx, "my_clock", (100.0, 100.0, 200.0, 100.0)
        );

        // WidgetWindow 自动包装你的内容，提供编辑/拖拽/边框能力
        cx.open_window(options, |window, cx| {
            let content = cx.new(|cx| ClockWidget::new(window, cx));
            let widget_window = cx.new(|_cx| widget_core::WidgetWindow::new(content));
            cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
        })
        .unwrap()
        .into()
    }
}

// 5. (关键!) 暴露标准的插件入口函数
pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(ClockPlugin)
}
```

### 3.3 重要：不要重复造轮子

以下功能由 `WidgetWindow` 统一提供，**不要在插件中重复实现**：

| 功能 | 由谁提供 |
|---|---|
| 编辑模式检测 (`UIState::is_edit_mode`) | WidgetWindow |
| 窗口样式切换 (`update_window_edit_mode`) | WidgetWindow |
| 编辑模式拖拽条 | WidgetWindow |
| 编辑模式边框高亮 | WidgetWindow |
| 窗口位置恢复 (`resolve_plugin_bounds`) | `default_widget_window_options` |
| 窗口选项样板 (`WindowOptions`) | `default_widget_window_options` |

### 3.4 高级：条件控制拖拽条

如果你的插件在某些状态下不希望显示拖拽条，覆盖 `show_drag_handle` 即可：

```rust
impl widget_core::WidgetContent for MyWidget {
    fn plugin_id(&self) -> &'static str { "my_widget" }

    fn show_drag_handle(&self) -> bool {
        // 例：全屏模式下不显示拖拽条
        !self.is_fullscreen
    }
}
```

## 4. 使用 CLI 安装插件

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

## 5. 插件数据持久化

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

## 6. 卸载插件
如果你不再需要这个插件，同样通过 CLI 卸载：
```bash
cargo run -p widget-cli -- plugin remove my_clock
```
