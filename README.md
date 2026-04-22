# Widget RS

基于 **Rust + Slint** 构建的桌面小部件系统，支持始终置顶、鼠标穿透、系统托盘驱动，以及插件化扩展。

---

## 快速开始

```bash
cargo run
```

右键系统托盘图标可切换控制面板显示/隐藏，或退出程序。

---

## 技术栈

| 层级 | 技术 |
|---|---|
| UI 渲染 | [Slint](https://slint.dev/) 1.8 |
| 系统托盘 | [tray-icon](https://github.com/tauri-apps/tray-icon) |
| 数据持久化 | serde_json + directories |
| Win32 交互 | windows-sys 0.52 |
| 原生窗口句柄 | raw-window-handle 0.6 |

---

## 项目结构

```
widget-rs/
├── build.rs                  # Slint 编译脚本
├── src/
│   ├── main.rs               # 程序入口，事件循环驱动
│   ├── window_manager.rs     # 窗口管理，任务栏隐藏核心逻辑
│   ├── tray.rs               # 系统托盘图标与菜单
│   ├── store.rs              # JSON 配置读写
│   └── plugin_manager.rs     # 插件目录扫描
└── ui/
    ├── index.slint           # UI 根模块导出
    ├── theme.slint           # 全局主题 Token
    ├── main_window.slint     # 主控制面板
    ├── sticky_widget.slint   # 便签悬浮窗
    ├── todo_widget.slint     # 待办悬浮窗
    └── components/           # 基础 UI 组件库
        ├── button.slint
        ├── card.slint
        ├── drag_handle.slint
        ├── todo_item.slint
        └── line_edit.slint
```

---

## 关键实现说明

### 1. 关闭主窗口不退出程序

程序使用 `slint::run_event_loop()`（而非绑定到主窗口的 `.run()`），配合拦截关闭事件：

```rust
wm.main_window().window().on_close_requested(|| {
    slint::CloseRequestResponse::HideWindow  // 隐藏而非退出
});
```

只有点击托盘菜单「Quit」才会调用 `slint::quit_event_loop()` 真正退出。

---

### 2. Win11 任务栏隐藏 — 三层保障方案

> **问题背景**：桌面悬浮小部件（no-frame 浮动窗口）不应出现在任务栏。
> 在 Windows 11 上，常规的 `WS_EX_TOOLWINDOW` 样式单独使用往往无效，
> 因为 Win11 Shell 在窗口第一次变为可见时就已经注册了任务栏按钮。

#### 失败的尝试

| 方案 | 失败原因 |
|---|---|
| 在 `show()` 前用 `EnumWindows` 设置 `WS_EX_TOOLWINDOW` | Slint 的 Win32 窗口在事件循环首次运行前尚未完全初始化 |
| `show()` 后立即设置 + `SW_HIDE`→`SW_SHOW` | Shell 已注册按钮；Slint 内部渲染循环会重新触发 `ShowWindow` 覆盖修改 |
| 用 `FindWindowW` 查找 HWND | 无边框窗口没有 OS 级 title，查找失败 |
| `raw-window-handle` 在事件循环启动前获取 HWND | 此时 `window_handle()` 返回 `None` |

#### 最终有效方案

```
事件循环启动（run_event_loop）
    └─ Timer::single_shot(500ms)  ← 在事件循环内延迟 500ms
           ├─ 通过 Weak<Component> → window_handle() 拿到精确 HWND
           ├─ CreateWindowExW 创建一个永远不显示的 dummy owner 窗口
           ├─ SetWindowLongPtr(popup, GWLP_HWNDPARENT, dummy_hwnd)
           ├─ SetWindowLongPtr(popup, GWL_EXSTYLE, WS_EX_TOOLWINDOW)
           └─ ShowWindow(SW_HIDE) → ShowWindow(SW_SHOWNOACTIVATE)
```

**三层保障的原理**：

1. **精确 HWND**：通过 `slint` 的 `raw-window-handle-06` feature 直接获取目标窗口句柄，避免 `EnumWindows` 误操作其他窗口

2. **隐藏的 dummy owner**（核心）：Windows 规则是「拥有可见任务栏按钮的 owner 的弹出窗口，其按钮跟随 owner」。创建一个 `STATIC` 类型的 1×1 像素窗口（永不显示），将其设为弹出窗口的 owner。由于 dummy owner 本身不在任务栏，owned 的弹出窗口也就不会出现在任务栏。

3. **Hide → ShowNoActivate 刷新**：通知 Win11 Shell 重新评估该窗口的任务栏状态。

```rust
// Cargo.toml
slint = { version = "1.8.0", features = ["raw-window-handle-06"] }
raw-window-handle = "0.6"
```

```rust
// 获取精确 HWND（只在事件循环内有效）
fn get_hwnd(window: &slint::Window) -> Option<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let slint_wh = window.window_handle();
    let raw_wh   = slint_wh.window_handle().ok()?;
    match raw_wh.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get() as isize),
        _ => None,
    }
}

// 创建永不显示的 dummy owner
let dummy = CreateWindowExW(0, "STATIC\0", "\0", WS_OVERLAPPED, 0,0,1,1, 0,0,0, null());

// 应用三层保障
SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, dummy);   // dummy owner
SetWindowLongPtrW(hwnd, GWL_EXSTYLE, WS_EX_TOOLWINDOW); // 工具窗口样式
ShowWindow(hwnd, SW_HIDE);
ShowWindow(hwnd, SW_SHOWNOACTIVATE);                // 强制 Shell 刷新
```

---

### 3. 系统托盘事件驱动

`tray-icon` 的 `MenuEvent` 通过 `slint::Timer` 在 Slint 事件线程内轮询，避免跨线程问题：

```rust
tray_timer.start(TimerMode::Repeated, Duration::from_millis(50), move || {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        // Toggle / Quit
    }
});
```

---

## 后续规划

- [ ] 便签/待办数据持久化绑定（写入 `store.rs` JSON）
- [ ] 鼠标穿透切换（winit 后端 `WS_EX_TRANSPARENT`）
- [ ] 插件系统：动态加载 `plugins/` 目录下的 `.slint` 文件
- [ ] 插件市场 UI
