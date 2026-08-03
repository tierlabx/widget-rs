# Widget RS 贡献与开发指南 (Contributing Guidelines)

欢迎贡献 Widget RS！作为标准的开源项目，为了保证代码质量、统一开发风格与高效协作，请在开发前仔细阅读以下规范。

## 1. 核心开发规则
- **UI 页面禁用 emoji**：仅在应用页面 (UI/界面渲染) 中禁止使用 emoji 图标，其他场景（如代码注释、提交信息、PR 描述及文档）可以正常使用。
- **AI 助理要求**：search 等待太久就主动关闭。

## 2. 开发工作流程 (Workflow)
1. **准备工作**：安装最新的 Rust 工具链（推荐稳定版）。
2. **分支管理**：基于 `main` 分支创建新分支进行开发。分支命名建议：`feat/xxx`, `fix/xxx`, `refactor/xxx`, `docs/xxx` 等。
3. **编码设计**：
   - 充分阅读现有代码，保持设计风格的一致性。
   - 添加新插件请参照 `plugins/` 目录下的示例（如 `sticky_plugin` 或 `todo_plugin`）。
4. **本地验证**：
   - **格式化**：提交前必须运行 `cargo fmt --all`，保持格式统一。
   - **静态检查**：运行 `cargo clippy --all-targets --all-features -- -D warnings`，原则上必须修复所有警告。
   - **测试**：运行 `cargo test` 确保没有破坏现有功能。
5. **提交 (Commit)**：
   - **必须**使用 [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) (约定式提交) 规范。
   - 例如：`feat: 增加桌面吸附功能`，`fix: 修复窗口大小调整失效的问题`。
   - 项目依赖 `release-plz` 进行自动化发版和 CHANGELOG 生成，不规范的提交将导致发版失败。
6. **Pull Request**：
   - 推送分支后，向主仓库的 `main` 分支发起 PR。
   - 关注 GitHub Actions CI 流水线状态，直至检查全部通过并等待代码 Review。

## 3. 代码架构与规范 (Code Standards)
- **Rust 习惯**：严格遵守 Rust 官方编码规范和惯用法（Idiomatic Rust）。
- **模块化架构**：
  - `crates/app`：主程序入口，负责托盘、窗口管理及生命周期。
  - `crates/core`：核心抽象和通用接口定义。
  - `crates/ui`：UI 界面组件。
  - `plugins/*`：所有的扩展插件。**强制要求插件分离 UI 渲染与状态逻辑**，通常拆分为 `model.rs`（数据与持久化）和 `view.rs`（UI 渲染层）。
  - 请严格遵循上述架构边界，避免循环依赖。
- **文档与注释**：
  - 公共函数、Trait 和 Struct 必须提供清晰的文档注释（使用 `///`）。
  - 复杂的业务逻辑内部需配以简明扼要的说明注释（使用 `//`）。
