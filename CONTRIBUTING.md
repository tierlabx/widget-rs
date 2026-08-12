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

为了配合 GitHub Actions 以及 `widget-cli release` 自动化发布工具，本项目**强制要求使用 Conventional Commits (约定式提交)**。

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

本项目采用插件化架构，开发一个新的桌面部件非常简单。为了保持代码可维护性，我们**强制要求在插件开发中采用 UI 与逻辑分离 (MVC 模式)**。

1. 在 `plugins/` 目录下新建一个 Cargo 包：`cargo new --lib plugins/my_plugin`
2. 按照分离规范组织代码结构：
   - `src/model.rs`: 纯数据结构定义、状态管理与持久化逻辑。
   - `src/view.rs`: 具体的 `GPUI` 渲染树实现 (实现 `Render` 和 `WidgetContent` Trait)。
   - `src/lib.rs`: 作为模块入口，暴露插件和统一装配。
3. 你的小部件 UI 结构体必须实现 `widget_core::WidgetContent` Trait，提供 `plugin_id()` 和 `drag_label()`，窗口级能力（编辑模式、拖拽、边框切换）由 `WidgetWindow` 统一管理，**禁止在插件内手动实现**。
4. 在 `lib.rs` 中实现 `widget_core::Plugin` Trait，`spawn_window` 使用 `WidgetWindow` 包装你的内容：
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
5. 暴露统一入口点：
   ```rust
   pub fn create_plugin() -> std::sync::Arc<dyn widget_core::Plugin> {
       std::sync::Arc::new(MyPlugin)
   }
   ```
6. 使用提供的命令行工具自动化注册插件：
   ```bash
   cargo run -p widget-cli -- plugin add my_plugin --path plugins/my_plugin
   ```

> **注意**: 编辑模式检测、拖拽条、窗口边框、`update_window_edit_mode` 等窗口级逻辑已由 `WidgetWindow` 容器统一封装，**不要**在插件中重复实现。

详细的 API 说明请参考 `docs/PLUGIN_DEVELOPMENT.md`。

## 📦 打包与构建

本项目使用 [cargo-packager](https://github.com/tauri-apps/cargo-packager) 进行应用程序的打包和分发。

如果你需要在本地构建安装包进行测试，请运行以下命令：
```bash
cargo packager --release
```
*(注意：首次使用需要先安装该工具，运行 `cargo install cargo-packager --locked`)*

期待你的第一个 PR！🚀
