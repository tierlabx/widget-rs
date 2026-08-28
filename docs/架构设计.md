# Widget RS - 架构设计文档 (Architecture Design)

## 1. 宏观系统架构

系统采用清晰的 **三层架构设计 (UI Layer - Controller Layer - Data Layer)**，并在控制层拓展了**插件机制沙箱 (Plugin System)**，以支持动态功能扩展和插件市场。

```mermaid
graph TD
    subgraph UI Layer - GPUI
        MW[Main Window .rs]
        WW["WidgetWindow&lt;T&gt; 容器"]
        SW[Sticky Widget]
        TW[Todo Widget]
        STW[Stretchly Widget]
        PW[Dynamic Plugin Widgets]
        Tray[System Tray Icon]
    end

    subgraph Controller Layer - Rust
        AppCore[App Core Manager]
        WM[Window Manager]
        EH[Event Handler / Tray Handler]
        PM[Plugin & Interpreter Manager]
        Sandbox[Wasm/Script Sandbox]
    end

    subgraph Data Layer - Rust and OS
        Store[Local Storage Manager]
        JSON[(Config & Data Files)]
        Plugins[(Local Plugin Assets)]
        OSAPI[(OS API / Win32)]
    end

    %% UI 组件关系
    SW --> WW
    TW --> WW
    STW --> WW
    PW --> WW

    %% UI to Controller
    MW -- User Interactions --> EH
    WW -- Drag/Edit Mode --> EH
    Tray -- Context Menu Actions --> EH

    %% Controller to UI
    EH -- Callbacks --> WM
    WM -- Updates Props/State --> MW
    WM -- Updates Props/State --> WW
    PM -- GPUI Component --> PW
    Sandbox -- Modifies Plugin UI --> PM

    %% Controller to OS API
    WM -- Window HitTest / Pos --> OSAPI

    %% Controller to Data
    EH -- Read/Write Req --> Store
    PM -- Load Plugin Files --> Plugins
    Store -- I/O --> JSON
```

## 2. 核心模块说明

### 2.1 UI Layer (表现层)
包含静态编译的 UI（主窗口、内置小部件）以及运行时动态解释的 UI。
* **职能**：定义界面的布局、颜色、动画、交互热区。
* **WidgetWindow 容器**：所有小部件（内置与第三方）统一通过 `WidgetWindow<T>` 容器包装。该容器封装了编辑模式检测、拖拽条渲染、窗口边框切换、`update_window_edit_mode` 调用等通用窗口行为。插件只需实现 `WidgetContent` trait 即可获得完整窗口能力。
* **动态组件 (Dynamic Plugin Widgets)**：通过 GPUI 组件体系或 WASM 加载执行的第三方界面。

### 2.2 Controller Layer (控制层)
基于 Rust 的核心业务运行枢纽。
* **App Core Manager & Window Manager**：核心事件调度与多窗口句柄管理。
* **Plugin & Interpreter Manager (新增)**：插件生命周期管理器。负责发现、下载（插件市场）、解压并注册第三方插件。未来通过 WASM 引擎或动态组件库解析执行。
* **Sandbox (新增)**：插件安全运行沙箱。限制第三方逻辑仅能调用公开安全的 Host 接口，隔离可能导致崩溃的代码。

### 2.3 Data Layer (数据层)
数据持久化封装与底层系统调用。
* **Local Storage Manager**：提供数据的结构化读写 API，同时也负责管理下载到本地的插件资源文件（`plugins/` 目录）。
* **单向数据流**：当 UI 发起变更，Controller 同步写入持久化存储并推送状态给其他实例。

## 3. 插件窗口层级模型

```
┌─────────────────────────────────────────┐
│  WidgetWindow<T>      (widget-core)     │  ← 框架统一管理
│  ├── 编辑模式检测 (UIState::is_edit_mode)  │
│  ├── 拖拽条 (drag handle, #00d992)       │
│  ├── 窗口边框切换 (edit mode border)      │
│  ├── update_window_edit_mode (WS_THICKFRAME) │
│  │                                      │
│  │  ┌───────────────────────────┐       │
│  │  │  T: WidgetContent         │       │  ← 插件只实现此层
│  │  │  ├── plugin_id()          │       │
│  │  │  ├── drag_label()         │       │
│  │  │  ├── show_drag_handle()   │       │
│  │  │  └── render() → 业务 UI   │       │
│  │  └───────────────────────────┘       │
│  │                                      │
└─────────────────────────────────────────┘
```

## 4. 事件循环与生命周期 (Lifecycle)
1. **初始化**：Rust `main()` 启动，读取本地 JSON 数据。
2. **插件装载**：Plugin Manager 扫描本地 `plugins/` 目录，初始化第三方插件并预加载其动态 UI 及 Wasm/脚本逻辑。
3. **窗口孵化**：各插件调用 `spawn_window`，通过 `WidgetWindow::new(content)` 统一包装后创建窗口。框架自动将窗口注册到 WindowManager。
4. **运行态**：
   * `WidgetWindow` 容器在每帧 `render()` 中读取 `UIState::is_edit_mode`，统一更新窗口样式并渲染拖拽条。
   * 插件的 `WidgetContent::render()` 只关注自身业务 UI 的渲染。
   * 插件 UI 产生的事件传递给 Sandbox，Sandbox 内部逻辑计算后通过 Host 暴露的接口请求 Rust 更新全局状态或弹窗。
5. **特殊交互状态（休眠/吸附/穿透）**：由系统接管，对原生与插件窗口一视同仁。
6. **销毁**：通知所有沙箱安全退出，执行全局数据落盘。
