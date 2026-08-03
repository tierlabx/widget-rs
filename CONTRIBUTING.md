# 贡献指南 (Contributing to Widget-RS)

感谢你考虑为 Widget-RS 做出贡献！本项目致力于构建一个基于 Rust 的现代桌面小部件平台，不论是提交 Bug 反馈、代码优化还是新增插件，我们都非常欢迎！

## 👨‍💻 开发准备

1. **环境配置**：请确保你的机器已安装最新版 [Rust](https://rustup.rs/) (>= 1.75)。
2. **格式化与 Lint**：我们在 `rustfmt.toml` 与 `clippy.toml` 中统一了代码风格和检查级别。请在提交代码前，务必在根目录运行以下命令进行自我检查：
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   ```

## 🛠 提交流程与规范 (约定式提交)

为了配合 GitHub Actions 以及 `release-plz` 自动化发布工具，本项目**强制要求使用 Conventional Commits (约定式提交)**。

提交信息格式要求如下：
```text
<type>(<scope>): <subject>
```

- **type** 必须是以下关键字之一：
  - `feat`: 新增功能（会触发次版本升级，并写入 Release Notes）
  - `fix`: 修复 Bug（会触发修订号升级，并写入 Release Notes）
  - `docs`: 仅修改文档 (如 README.md)
  - `style`: 调整代码格式（不影响功能）
  - `refactor`: 重构代码（非新增功能也非修复 bug）
  - `perf`: 性能优化
  - `test`: 新增或修改测试
  - `chore`: 构建过程或辅助工具变动
- **scope** (可选): 指明改动影响的模块，如 `ui`, `core`, `sticky`, `todo`。
- **subject**: 简短的改动描述。

> ✅ 正确示例: `feat(sticky): add color picker support`
> ❌ 错误示例: `fixed some bugs` 或 `更新便签功能`

## 🧩 如何开发一个新的插件 (Plugin)

本项目采用插件化架构，开发一个新的桌面部件非常简单：

1. 在 `plugins/` 目录下新建一个 Cargo 包：`cargo new --lib plugins/my_plugin`
2. 让你的插件入口实现 `widget_core::Plugin` Trait (提供 `spawn_window`, `on_load` 等方法)。
3. 在插件代码根目录（通常是 `lib.rs`）暴露统一入口点：
   ```rust
   pub fn create_plugin() -> std::sync::Arc<dyn widget_core::Plugin> {
       std::sync::Arc::new(MyPlugin)
   }
   ```
4. 使用提供的命令行工具自动化注册插件：
   ```bash
   cargo run -p widget-cli -- plugin add my_plugin --path plugins/my_plugin
   ```

如果你在开发中遇到任何架构设计上的疑问，请参考 `docs/PRD.md` 和 `README.md` 中的架构图。

期待你的第一个 PR！🚀
