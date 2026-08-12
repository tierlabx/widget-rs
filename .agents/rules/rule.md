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
- **插件开发规范**：
  - 所有小部件插件**必须**使用 `WidgetWindow<T>` 容器包装，**禁止**在插件的 `view.rs` 中手动实现编辑模式检测、拖拽条渲染、窗口边框切换、`update_window_edit_mode` 等窗口级逻辑。
  - 插件的 UI 结构体必须实现 `widget_core::WidgetContent` trait（提供 `plugin_id()` 和 `drag_label()`）。
  - `spawn_window` 必须使用 `widget_core::default_widget_window_options()` 创建窗口选项，并通过 `WidgetWindow::new(content)` 包装内容。
  - 如有特殊需求（如条件隐藏拖拽条），通过覆盖 `show_drag_handle()` 方法实现，不要自行渲染拖拽条。

## 3. 工作流与自动化规范
- **约定式提交 (Conventional Commits)**：
  - 帮你生成或执行 Git 提交时，提交信息**必须**采用 Conventional Commits 格式（如 `feat: xxx`, `fix: xxx`, `docs: xxx`, `chore: xxx`）。
  - 严格的提交格式是 `widget-cli release` 自动化版本发布和更新日志生成的基础。
- **本地预检意识**：
  - 在完成功能代码编写后、回复用户前，建议主动（或提示用户）执行 `cargo fmt --all` 和基本的 `cargo check`，确保交付的代码是合规且无明显编译错误的。
- **开发前防重复分析机制**：
  - 在开发或设计新功能前，**必须**先对代码库进行充分的搜索与分析，确认是否已经存在相似的模块或机制（如项目中的 `widget-cli` 已实现插件自动注册逻辑，`WidgetWindow` 已封装窗口通用行为）。
  - **严禁重复造轮子**。必须优先复用和增强已有机制，绝对不能忽视代码库中已经存在的解决方案而自行重新设计。