# Widget-RS 🎨

[![CI](https://github.com/tierlabx/widget-rs/actions/workflows/packager.yml/badge.svg)](https://github.com/tierlabx/widget-rs/actions/workflows/packager.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

基于 **Rust + GPUI** 构建的轻量级、高性能桌面小部件系统，支持始终置顶、鼠标穿透、原生边缘磁吸、系统托盘驱动，以及完善的插件化扩展能力。

---

## 🌟 特性 (Features)

- **高性能渲染**: 使用 `GPUI` 现代渲染引擎，利用 GPU 极速渲染。
- **动态插件系统**: 将小部件解耦为独立的插件 (如 `Sticky`, `Todo` 等)，方便扩展。
- **原生边缘吸附**: 基于 `Win32 API` 的多屏幕边缘磁力吸附，提供极其流畅的拖拽手感。
- **免打扰模式**: 支持单点开启 “鼠标穿透” 与 “总在最前”，完美融入桌面背景。
- **现代化工程化**: 完善的 CI/CD 流水线，统一的代码 Lint 规则。

---

## 🏗 架构设计 (Architecture)

Widget-RS 采用分层与插件化架构，保证核心稳定性的同时允许极大的扩展自由：

<p align="center">
  <img src="assets/architecture.svg" alt="Widget-RS Architecture" width="100%">
</p>

---

## 🚀 快速开始

### 1. 环境准备
确保你已经安装了较新版本的 [Rust](https://rustup.rs/) (建议 1.75+)。

### 2. 构建与运行
```bash
git clone https://github.com/your-username/widget-rs.git
cd widget-rs

# 运行调试版本
cargo run

# 构建发布版本
cargo packager --release
```

### 3. 操作指南
- **排版模式**: 右键系统托盘图标，点击 **“控制面板”**，开启排版模式即可拖拽小部件、感受边缘吸附。
- **配置保存**: 所有的小部件位置、大小及开启状态都会持久化到本地配置文件中，下次启动自动还原。

---

## 🧩 插件管理 (CLI)

Widget-RS 提供了一个强大的命令行工具 `widget-cli`，用于源码级别的插件一键安装与卸载，确保你在享受原生高性能渲染的同时，轻松扩展功能。

### 安装与卸载插件
```bash
# 添加一个本地插件 (自动注入 Cargo.toml 与 源码注册表)
cargo run -p widget-cli -- plugin add <插件名称> --path <本地路径>
# 示例：
cargo run -p widget-cli -- plugin add my_clock --path ../plugins/my_clock

# 卸载一个已安装的插件
cargo run -p widget-cli -- plugin remove <插件名称>
# 示例：
cargo run -p widget-cli -- plugin remove sticky_plugin
```

### 开发你自己的插件
想要开发自己的插件，你只需要：
1. 创建一个新的 Rust 库 (`cargo new --lib plugins/my_plugin`)
2. 引入 `widget-core` 依赖
3. 遵循 **UI与逻辑分离规范**，建立 `model.rs` 存放数据逻辑，`view.rs` 存放渲染代码。
4. 在 `view.rs` 中实现 `widget_core::WidgetContent` Trait（只需提供 `plugin_id` 和 `drag_label`），窗口级能力（编辑模式、拖拽、边框）由 `WidgetWindow` 容器自动提供。
5. 在 `lib.rs` 中实现 `widget_core::Plugin` Trait，使用 `WidgetWindow` 包装你的内容：
```rust
fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
    let options = widget_core::default_widget_window_options(cx, "my_plugin", (100.0, 100.0, 300.0, 300.0));
    cx.open_window(options, |window, cx| {
        let content = cx.new(|cx| MyWidget::new(window, cx));
        let widget_window = cx.new(|_cx| widget_core::WidgetWindow::new(content));
        cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
    }).unwrap().into()
}
```
6. 使用 CLI 将其添加到主程序中编译运行！

> 详细的 API 说明和完整示例请参考 [插件开发指南](docs/PLUGIN_DEVELOPMENT.md)。

---

## 🔖 版本发布 (Release)

Widget-RS 内置了基于 [Conventional Commits](https://www.conventionalcommits.org/) 的版本发布工具，自动计算版本号、更新 `Cargo.toml`、生成 `CHANGELOG.md`、提交并推送 tag。

```bash
# 自动根据 commit 历史计算版本号并发布
cargo run -p widget-cli -- release

# 预览下一个版本（不做任何修改）
cargo run -p widget-cli -- release --dry-run

# 手动指定版本号
cargo run -p widget-cli -- release --version 0.2.0
```

推送 tag 后，GitHub Actions 会自动触发打包并创建 GitHub Release。

## 🛠 技术栈

| 领域 | 核心技术 | 说明 |
| :--- | :--- | :--- |
| **渲染引擎** | [GPUI](https://gpui.rs/) | 极速、现代的 Rust UI 框架 |
| **UI 组件库**| gpui-component | 构建标准 UI 控件 |
| **底层交互** | windows-sys 0.52 | 原生窗口消息拦截、边缘吸附处理 |
| **系统托盘** | tray-icon | 跨平台系统托盘支持 |
| **工程构建** | widget-cli, GitHub Actions | 内置版本发布工具与 CI/CD 自动打包 |

---

## 🤝 参与贡献 (Contributing)

我们非常欢迎开发者参与贡献！无论你是想修复 Bug、开发新的小部件插件、还是改进文档，请参阅 [贡献指南 CONTRIBUTING.md](CONTRIBUTING.md) 了解详细的开发流程与规范。

## 📄 开源协议

本项目基于 MIT 协议开源。详细信息请参阅 [LICENSE](LICENSE) 文件。
