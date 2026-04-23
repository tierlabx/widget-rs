# Widget RS - 技术设计文档 (Technical Design)

## 1. 技术栈选型
* **核心语言**: Rust (保证内存安全与极致性能)。
* **GUI 框架**: `gpui` (高性能 GPU 加速 UI，原生支持组件化与复杂状态管理)。
* **系统托盘**: `tray-icon` (跨平台系统托盘与右键菜单支持)。
* **窗口管理增强**: 结合 GPUI 提供的 `Window` API，必要时直接操作底层的原生窗口句柄（以便实现高级的边缘吸附、透明度动态调节以及鼠标穿透）。
* **序列化与存储**: `serde`, `serde_json` (处理本地配置文件和数据文件的读写)。
* **动态扩展/插件化**: 基于 `gpui-component` 的组件系统与 WASM 沙箱。

## 2. 关键技术方案

### 2.1 GPUI 组件交互
* **设计整合**：UI 结构与样式使用 GPUI 的 Fluent Builder 模式在 Rust 代码中直接定义，或者封装为可复用的组件。
* **桥接逻辑**：在 Rust 中通过 `gpui_component::Root::new()` 等方法实例化组件。事件监听直接在渲染闭包中通过 `on_click` 等方法绑定。

### 2.2 多窗口支持与无边框特性
* 小部件（便签、待办）要求是无边框独立悬浮窗。
* **实现方案**：在 GPUI 的 WindowOptions 中设置 `titlebar: None` 以及 `window_background: WindowBackgroundAppearance::Transparent`。
* 拖拽实现：由于去除了原生标题栏，可以在 GPUI 中捕获鼠标拖拽事件，并调用平台原生 API 或 GPUI 的拖拽扩展执行窗口拖拽。

### 2.3 高级窗口交互
* **始终置顶 (Always on Top)**：在创建小部件窗口时，将对应平台窗口属性（`always_on_top`）设置为 true。
* **边缘吸附与透明度调节**：
  * **边缘吸附**：需要 Rust 层监听窗口的移动事件。当窗口坐标接近显示器边缘的阈值（如 20px）时，自动纠正其坐标，使其贴合边缘。
  * **透明度控制**：窗口静置或吸附时，定时器触发状态切换，降低 GPUI 视图元素的 `opacity` 属性（例如 0.5）。当检测到 hover 时，平滑恢复到 1.0。
* **鼠标穿透 (Mouse Passthrough)**：为实现小部件不干扰日常操作，需在 Rust 控制层通过获取底层原生窗口句柄（如 `winit` 的 `set_cursor_hittest(false)`）动态开启/关闭窗口的鼠标事件拦截。开启穿透后，拖拽区将暂时失效，因此须通过系统托盘菜单等外部控制流提供关闭穿透的入口。

### 2.4 插件市场与动态扩展
* **组件化插件**：为了支持第三方插件，通过暴露标准的 GPUI Plugin 接口，并探索使用动态链接库或 WASM 将 UI 与逻辑下发到本地执行。
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
gpui = { git = "https://github.com/zed-industries/zed" }
gpui-component = "0.1.0"
tray-icon = "0.14" 
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
├── ui/                       # UI 组件层 (UI Layer)
│   ├── theme.rs              # 全局样式规范（颜色、间距、字体等，对应 VoltAgent 规范）
│   ├── main_window.rs        # 主管理面板界面
│   ├── sticky_widget.rs      # 便签小部件界面
│   ├── todo_widget.rs        # 待办小部件界面
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
        ├── ui.rs             # 插件的 UI 组件代码
        ├── logic.wasm        # 插件的编译后沙箱逻辑
        └── manifest.json     # 插件元数据（名称、版本、所需系统权限声明）
```
