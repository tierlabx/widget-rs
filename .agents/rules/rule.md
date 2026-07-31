---
trigger: always_on
---

# Widget RS Agent Rules

作为参与 `widget-rs` 开源项目开发的 AI Agent，你必须在整个交互与开发过程中严格遵守以下行为准则与规范：

## 1. 强制规则
- **UI 页面禁用 emoji**：仅在开发应用界面（UI）及前端渲染部分时，禁止使用 emoji 图标。在其他场景（如与用户对话、编写文档、Commit Message、普通代码注释等）下允许使用。
- **搜索防挂死**：执行 search（或相关检索操作）时，如果等待时间太久，必须主动关闭或终止该操作，不要让流程卡死。

## 2. 编码与代码规范
- **Rust 标准**：遵循 Idiomatic Rust 风格。编写的任何代码必须能够通过 `cargo fmt` 格式化，并且通过 `cargo clippy` 检查。
- **模块化与注释**：
  - 遵循项目的 `app`、`core`、`ui` 及 `plugins` 隔离架构。
  - 重要的业务逻辑、开放的 API（public structs/traits/functions）必须添加标准的 Rust 文档注释（`///`）。

## 3. 工作流与自动化规范
- **约定式提交 (Conventional Commits)**：
  - 帮你生成或执行 Git 提交时，提交信息**必须**采用 Conventional Commits 格式（如 `feat: xxx`, `fix: xxx`, `docs: xxx`, `chore: xxx`）。
  - 这是由于项目接入了 `release-plz`，严格的提交格式是自动化版本发布和更新日志生成的基础。
- **本地预检意识**：
  - 在完成功能代码编写后、回复用户前，建议主动（或提示用户）执行 `cargo fmt --all` 和基本的 `cargo check`，确保交付的代码是合规且无明显编译错误的。