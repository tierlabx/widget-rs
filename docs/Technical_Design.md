# Widget RS - 技术设计文档 (Technical Design)

## 1. 技术栈选型
* **核心语言**: Rust (保证内存安全与极致性能)。
* **GUI 框架**: `slint` (声明式 UI，支持复杂现代动画、发光特效，且纯原生渲染，底层基于 Skia / FemtoVG)。
* **系统托盘**: `tray-icon` (跨平台系统托盘与右键菜单支持)。
* **窗口管理增强**: 结合 Slint 提供的 `Window` API，必要时直接操作底层的 `winit` 窗口句柄（以便实现高级的边缘吸附、透明度动态调节以及鼠标穿透）。
* **序列化与存储**: `serde`, `serde_json` (处理本地配置文件和数据文件的读写)。
* **动态扩展/插件化**: `slint::interpreter` (动态加载 `.slint` UI) + `wasmtime` / `rhai` (为插件逻辑提供安全沙箱，作为备选扩展技术)。

## 2. 关键技术方案

### 2.1 Slint UI 与 Rust 后端交互
* **设计分离**：所有的 UI 结构、样式（颜色、阴影、圆角）、Hover 动画均写在 `.slint` 文件中。
* **桥接逻辑**：在 Rust `main.rs` 或专属控制器中，通过 Slint 生成的 Rust struct (例如 `MainWindow::new()`) 进行实例化。Rust 后端通过 `on_xxx` 闭包监听 UI 事件（如按钮点击），通过 `set_xxx` 或 `invoke_xxx` 更新 UI 数据状态。

### 2.2 多窗口支持与无边框特性
* 小部件（便签、待办）要求是无边框独立悬浮窗。
* **实现方案**：在 Slint 的 Window 属性中设置 `no-frame: true;` 以及 `background: transparent;`。这需要底层的窗口系统支持透明窗口层。
* 拖拽实现：由于去除了原生标题栏，必须在 Slint 中实现一个 `TouchArea` 作为拖拽把手，并在触发时调用 Rust 暴露的回调来请求操作系统执行窗口拖拽 (`window.window().drag_window()`)。

### 2.3 高级窗口交互
* **始终置顶 (Always on Top)**：在创建小部件窗口时，将对应平台窗口属性（`always_on_top`）设置为 true。
* **边缘吸附与透明度调节**：
  * **边缘吸附**：需要 Rust 层监听窗口的移动事件。当窗口坐标接近显示器边缘的阈值（如 20px）时，自动纠正其坐标，使其贴合边缘。
  * **透明度控制**：窗口静置或吸附时，Rust 定时器或 Slint 定时器触发状态切换，降低 Slint 主元素的 `opacity` 属性（例如 0.5）。当 `TouchArea` 检测到 `has-hover` 时，通过动画属性（`animate opacity { duration: 200ms; }`）平滑恢复到 1.0。
* **鼠标穿透 (Mouse Passthrough)**：为实现小部件不干扰日常操作，需在 Rust 控制层通过获取底层原生窗口句柄（如 `winit` 的 `set_cursor_hittest(false)`）动态开启/关闭窗口的鼠标事件拦截。开启穿透后，拖拽区将暂时失效，因此须通过系统托盘菜单等外部控制流提供关闭穿透的入口。

### 2.4 插件市场与动态扩展
* **动态 UI 解析**：为了支持第三方插件，主程序打包时会包含 `slint::interpreter` 解释器模块。插件开发者只需提供纯文本的 `.slint` 界面文件，主程序可以在运行时动态将其渲染为原生小部件，避免了重新编译整个 Rust 程序的成本。
* **逻辑运行沙箱**：考虑到第三方插件的安全性，插件的业务代码不能直接编译为原生库。初期可以提供基于 REST/Local Socket 的跨进程通讯接口；中后期推荐集成 WebAssembly 沙箱 (`wasmtime`) 或嵌入式脚本引擎（如 `rhai`），插件开发者可使用 JS/TS/Rust 编译为 Wasm 并在沙箱内安全调用宿主（Host）提供的公开 API（如弹窗、发起网络请求等）。

### 2.5 本地存储系统
* 使用系统标准的应用数据目录 (`std::env` 获取，如 `%APPDATA%/WidgetRS/` 或 `~/.config/WidgetRS/`) 存放数据。
* 拆分文件：
  * `config.json`：存储全局配置。
  * `plugins/` 目录：存放本地安装的第三方插件包（包含 `.slint` UI 和 `.wasm` / `.rhai` 脚本文件）。
* **保存策略**：每次数据在 Rust 层发生变更时，采用防抖机制落盘。

## 3. 依赖包预估 (Cargo.toml)
```toml
[dependencies]
slint = "1.8" 
tray-icon = "0.14"
tao = "0.28" 
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
directories = "5.0"
# 以下为插件系统可选依赖
# wasmtime = "18.0" 或 rhai = "1.17"
```

## 4. 项目结构设计 (Project Structure Design)

项目将采用标准的 Cargo 目录结构，同时将 UI 资源（Slint）、核心逻辑和插件沙箱进行清晰的物理隔离：

```text
widget-rs/
├── Cargo.toml                # 项目全局依赖与工作区配置
├── build.rs                  # 构建脚本（负责在编译期间处理静态 .slint 文件）
├── ui/                       # 静态表现层 (UI Layer)
│   ├── theme.slint           # 全局样式规范（颜色、间距、字体等，对应 VoltAgent 规范）
│   ├── main_window.slint     # 主管理面板界面
│   ├── sticky_widget.slint   # 便签小部件界面
│   ├── todo_widget.slint     # 待办小部件界面
│   └── components/           # 复用的 UI 组件库（如自定义按钮、输入框、状态卡片）
├── src/                      # 核心控制与数据层 (Controller & Data Layer)
│   ├── main.rs               # 程序入口与主事件循环挂载
│   ├── app.rs                # App Core Manager (核心状态与生命周期调度)
│   ├── window_manager.rs     # 窗口控制器（多窗口句柄管理、边缘吸附、鼠标穿透拦截）
│   ├── tray.rs               # 系统托盘图标与右键事件处理
│   ├── store.rs              # 本地数据持久化（读写 JSON 配置和数据）
│   ├── plugin_manager.rs     # 插件市场交互、下载与动态 UI 加载管理器
│   └── sandbox.rs            # 插件安全运行沙箱（Wasm / 脚本解释器交互层）
└── plugins/                  # [运行期自动生成] 本地安装的插件资源目录
    └── example_plugin/       # 示例插件目录
        ├── ui.slint          # 插件的动态 UI 模板
        ├── logic.wasm        # 插件的编译后沙箱逻辑
        └── manifest.json     # 插件元数据（名称、版本、所需系统权限声明）
```
