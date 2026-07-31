# Widget RS

基于 **Rust + GPUI** 构建的桌面小部件系统，支持始终置顶、鼠标穿透、系统托盘驱动，以及插件化扩展。

---

## 快速开始

```bash
cargo run
```

右键系统托盘图标可切换控制面板显示/隐藏，或退出程序。

---

## 技术栈

| 层级         | 技术                                                 |
| ------------ | ---------------------------------------------------- |
| UI 渲染      | [GPUI](https://gpui.rs/) + gpui-component            |
| 系统托盘     | [tray-icon](https://github.com/tauri-apps/tray-icon) |
| 数据持久化   | serde_json + directories                             |
| Win32 交互   | windows-sys 0.52                                     |
| 原生窗口句柄 | raw-window-handle 0.6                                |

---

## 项目结构

```
widget-rs/
├── crates/
│   ├── app/                  # 应用程序主入口及系统托盘集成
│   │   └── src/
│   │       ├── main.rs
│   │       ├── plugin_manager.rs
│   │       ├── store.rs
│   │       ├── tray.rs
│   │       └── window_manager.rs
│   ├── core/                 # 核心业务逻辑与状态管理
│   │   └── src/
│   │       └── lib.rs
│   └── ui/                   # GPUI 组件与界面层
│       └── src/
│           ├── lib.rs
│           ├── main_window.rs
│           └── components/
│               ├── badge.rs
│               ├── button.rs
│               ├── card.rs
│               └── mod.rs
```

---

## 参与贡献 (Contributing)

欢迎提交 Issue 和 Pull Request！为了保证代码质量，请遵循以下开发规范：

### 1. 提交流程与规范
本项目接入了 `release-plz` 自动发版工具，因此**必须**使用 [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) (约定式提交) 规范。
提交信息示例：
- `feat: 新增便签插件`
- `fix(ui): 修复窗口大小调整失效的问题`

### 2. 代码规范与 Git Hook
提交代码前，请确保能够通过 `cargo fmt` 和 `cargo clippy` 的检查。
本项目使用原生的 Git Hook 来自动在提交前检查代码格式与 Commit Message 规范。

在您首次 Clone 本项目后，请在项目根目录运行以下命令以激活 Git Hook：
```bash
git config core.hooksPath .githooks
```
执行后，每次执行 `git commit` 时，Git 都会自动检查您的代码格式与提交信息规范。
