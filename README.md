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

