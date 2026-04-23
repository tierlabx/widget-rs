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

| 层级 | 技术 |
|---|---|
| UI 渲染 | [GPUI](https://gpui.rs/) + gpui-component |
| 系统托盘 | [tray-icon](https://github.com/tauri-apps/tray-icon) |
| 数据持久化 | serde_json + directories |
| Win32 交互 | windows-sys 0.52 |
| 原生窗口句柄 | raw-window-handle 0.6 |

---

## 项目结构

```
widget-rs/
└── ui/
    ├── mod.rs                # UI 根模块导出
    ├── theme.rs              # 全局主题 Token
    ├── main_window.rs        # 主控制面板
    ├── sticky_widget.rs      # 便签悬浮窗
    ├── todo_widget.rs        # 待办悬浮窗
    └── components/           # 基础 UI 组件库
        ├── button.rs
        ├── card.rs
        ├── drag_handle.rs
        ├── todo_item.rs
        └── line_edit.rs
```

---

## 关键实现说明

### 1. 关闭主窗口不退出程序

程序使用 GPUI 的全局应用状态，配合托盘菜单控制窗口显示/隐藏，而不是直接退出：

```rust
// 隐藏而非退出
cx.update_global::<WindowManager, _>(|wm, cx| {
    wm.toggle_main_window(cx);
});
```

只有点击托盘菜单「Quit」才会调用 `cx.quit()` 真正退出。

---

### 2. Win11 任务栏隐藏 — 三层保障方案

> **问题背景**：桌面悬浮小部件（no-frame 浮动窗口）不应出现在任务栏。
> 在 Windows 11 上，常规的 `WS_EX_TOOLWINDOW` 样式单独使用往往无效，
> 因为 Win11 Shell 在窗口第一次变为可见时就已经注册了任务栏按钮。

#### 失败的尝试

| 方案 | 失败原因 |
|---|---|
| 在 `show()` 前用 `EnumWindows` 设置 `WS_EX_TOOLWINDOW` | GPUI 的 Win32 窗口在事件循环首次运行前尚未完全初始化 |
| `show()` 后立即设置 + `SW_HIDE`→`SW_SHOW` | Shell 已注册按钮；GPUI 内部渲染循环会重新触发 `ShowWindow` 覆盖修改 |
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

1. **精确 HWND**：通过 GPUI 提供的原生窗口句柄获取目标窗口句柄，避免 `EnumWindows` 误操作其他窗口

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

`tray-icon` 的 `MenuEvent` 通过 GPUI 的后台异步任务轮询，避免跨线程问题：

```rust
cx.spawn(async move |cx| {
    loop {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            // Toggle / Quit
        }
        cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
    }
}).detach();
```

---

## 后续规划

- [ ] 便签/待办数据持久化绑定（写入 `store.rs` JSON）
- [ ] 鼠标穿透切换（winit 后端 `WS_EX_TRANSPARENT`）
- [ ] 插件系统：动态加载和管理独立编译的外部组件或脚本
- [ ] 插件市场 UI
