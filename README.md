# Widget-RS 🎨

[![CI](https://github.com/your-username/widget-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/widget-rs/actions/workflows/ci.yml)
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
cargo build --release
```

### 3. 操作指南
- **排版模式**: 右键系统托盘图标，点击 **“控制面板”**，开启排版模式即可拖拽小部件、感受边缘吸附。
- **配置保存**: 所有的小部件位置、大小及开启状态都会持久化到本地配置文件中，下次启动自动还原。

---

## 🛠 技术栈

| 领域 | 核心技术 | 说明 |
| :--- | :--- | :--- |
| **渲染引擎** | [GPUI](https://gpui.rs/) | 极速、现代的 Rust UI 框架 |
| **UI 组件库**| gpui-component | 构建标准 UI 控件 |
| **底层交互** | windows-sys 0.52 | 原生窗口消息拦截、边缘吸附处理 |
| **系统托盘** | tray-icon | 跨平台系统托盘支持 |
| **工程构建** | release-plz, GitHub Actions | 自动化版本发布与 CI/CD 测试 |

---

## 🤝 参与贡献 (Contributing)

我们非常欢迎开发者参与贡献！无论你是想修复 Bug、开发新的小部件插件、还是改进文档，请参阅 [贡献指南 CONTRIBUTING.md](CONTRIBUTING.md) 了解详细的开发流程与规范。

## 📄 开源协议

本项目基于 MIT 协议开源。详细信息请参阅 [LICENSE](LICENSE) 文件。
