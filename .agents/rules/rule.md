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
  - **单文件行数限制**：任何 `.rs` 文件不应超过 **400 行**。超出时必须按职责拆分为子模块。
- **插件开发规范**：
  - 所有小部件插件**必须**使用 `WidgetWindow<T>` 容器包装，**禁止**在插件的 `view.rs` 中手动实现编辑模式检测、拖拽条渲染、窗口边框切换、`update_window_edit_mode` 等窗口级逻辑。
  - 插件的 UI 结构体必须实现 `widget_core::WidgetContent` trait（提供 `plugin_id()` 和 `drag_label()`）。
  - `spawn_window` 必须使用 `widget_core::default_widget_window_options()` 创建窗口选项，并通过 `WidgetWindow::new(content)` 包装内容。
  - 如有特殊需求（如条件隐藏拖拽条），通过覆盖 `show_drag_handle()` 方法实现，不要自行渲染拖拽条。
- **插件设置弹窗规范**：
  - 所有带有独立设置弹窗的插件，必须在 `Plugin::build_settings_window` 中使用 `widget_core::default_settings_window_options(cx, initial_size)` 创建窗口。
  - 设置页面**必须**使用 `widget_core::render_settings_shell(title, content)` 统一包装，禁止在插件内手写标题栏拖拽逻辑、关闭按钮或外层滚动容器。
  - 设置页内的区块标题与卡片**必须**使用 `widget_core::settings_section_header()` 和 `widget_core::settings_card()`，确保全局设置弹窗视觉风格与交互行为 100% 一致。

## 3. 控制面板 UI 开发规范
- **模块化文件结构**（`crates/ui/src/`）：
  - `main_window.rs` — 仅包含 `MainWindow` 结构体和 `Render` impl，负责组装 Layout。
  - `layout.rs` — 通用布局函数：`page_header()`、`section_title()`、`settings_card()`、`settings_row()` 等。
  - `titlebar.rs` — 标题栏渲染逻辑。
  - `sidebar.rs` — 侧边导航渲染逻辑。
  - `update.rs` — 更新检查/下载逻辑及 `UpdateStatus` 枚举。
  - `pages/*.rs` — 每个页面一个文件（`dashboard.rs`、`widgets.rs`、`settings.rs`）。
  - `components/*.rs` — 可复用 UI 组件（`badge`、`button`、`card`、`toggle` 等）。
- **统一 Layout 滚动**：
  - 滚动由 `main_window.rs` 的 Layout 层统一处理（`overflow_hidden + min_h_0 + overflow_y_scroll`）。
  - **禁止**在页面文件（`pages/*.rs`）中自行添加 `overflow_y_scroll` 或 `min_h_0`。
  - 页面函数只返回内容元素，不关心滚动和外层容器。
- **复用通用组件**：
  - 页面标题**必须**使用 `layout::page_header(title, subtitle)`，禁止手写 `text_3xl + font_weight(BOLD)` 样板。
  - 区块标题**必须**使用 `layout::section_title(title)`。
  - 设置卡片**必须**使用 `layout::settings_card()` + `layout::settings_row()`。
  - Toggle 开关**必须**使用 `components::toggle::toggle_switch()`。
- **新增页面流程**：
  1. 在 `pages/` 下新建文件，实现页面内容函数。
  2. 在 `pages/mod.rs` 中声明模块。
  3. 在 `NavPage` 枚举中添加变体。
  4. 在 `sidebar.rs` 中添加导航项。
  5. 在 `main_window.rs` 的 `Render` match 中添加路由。
  6. **不需要**处理滚动、页面容器或 padding — Layout 已统一处理。

## 4. 工作流与自动化规范
- **约定式提交 (Conventional Commits)**：
  - 帮你生成或执行 Git 提交时，提交信息**必须**采用 Conventional Commits 格式（如 `feat: xxx`, `fix: xxx`, `docs: xxx`, `chore: xxx`）。
  - 严格的提交格式是 `widget-cli release` 自动化版本发布和更新日志生成的基础。
- **本地预检意识**：
  - 在完成功能代码编写后、回复用户前，建议主动（或提示用户）执行 `cargo fmt --all` 和基本的 `cargo check`，确保交付的代码是合规且无明显编译错误的。
- **开发前防重复分析机制**：
  - 在开发或设计新功能前，**必须**先对代码库进行充分的搜索与分析，确认是否已经存在相似的模块或机制（如项目中的 `widget-cli` 已实现插件自动注册逻辑，`WidgetWindow` 已封装窗口通用行为）。
  - **严禁重复造轮子**。必须优先复用和增强已有机制，绝对不能忽视代码库中已经存在的解决方案而自行重新设计。

## 5. 核心窗口能力保护规范 (Core Window Behaviors)
- **Win+D 桌面常驻特性**：
  - 桌面小组件的核心定位是“融入桌面”，用户按 `Win+D`（显示桌面）时，普通小组件**必须始终与桌面一同保留**，绝不能随普通应用程序被最小化或隐藏。
  - 实现机制：必须在 `apply_plugin_window_styles` 中通过 `GWLP_HWNDPARENT` 将小组件挂载到系统 `Progman`（桌面窗口）。**严禁**在后续重构或修改中漏掉或移除此桌面绑定逻辑。
- **置顶与取消置顶标准**：
  - 置顶必须使用 `HWND_TOPMOST`（`-1`），取消置顶**必须**使用 `HWND_NOTOPMOST`（`-2`），**严禁**误用 `HWND_BOTTOM`（`1`）。
  - 所有置顶/取消置顶操作必须统一调用 `widget_core::set_window_always_on_top(hwnd, is_top)`，避免重复手写 Win32 常量。
- **透明/磨砂直通桌面通道**：
  - 必须保留 `DwmExtendFrameIntoClientArea` 与 `SetWindowCompositionAttribute`，确保 DirectComposition 渲染管线透明通道直通壁纸，无原生白边与闪烁。