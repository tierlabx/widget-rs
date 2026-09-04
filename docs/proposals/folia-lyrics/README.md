# 提案 PROP-001：沉浸式桌面流光歌词小部件 (Folia Lyrics)

> **当前状态**：方案评审中 (Review / RFC)  
> **灵感来源**：[chthollyphile/folia-major](https://github.com/chthollyphile/folia-major)（高质感沉浸式动效歌词播放器）  
> **目标架构**：Rust + GPUI 原生桌面渲染引擎  

---

## 1. 提案背景与产品定位

### 1.1 背景痛点
桌面音乐歌词通常分为两种极端：
- **简陋型**：仅有一行半透明悬浮文字或简单 LRC 滚动，视觉呆板、无节奏动效与舞台沉浸感。
- **庞大型**：基于 Electron/WebGL（如 Folia-Major 原版），虽然视效惊艳（逐字动效、流体背景），但常驻内存动辄 350MB~600MB，且作为独立应用窗口容易被系统 `Win+D`（显示桌面）隐藏，难以自然融入桌面壁纸生态。

### 1.2 本组件核心定位
将 Folia-Major 顶级的“视觉舞台感”与 `widget-rs` 的“轻量、常驻、高性能”相结合：
- **超轻量化运行**：利用 GPUI + DirectComposition 硬件加速，内存控制在 **30MB~50MB** 以内，CPU 闲置占用低于 0.3%，支持 120Hz 高刷屏平滑帧率。
- **双模架构 (Listener + Standalone)**：
  - **全局媒体监听模式 (SMTC Listener，主力场景)**：零配置、免登录。用户使用网易云、QQ音乐、Spotify、Apple Music、Foobar2000 或浏览器播放音乐时，小组件自动捕获曲目与进度，秒级完成逐字歌词匹配并呈现沉浸式流光舞台。
  - **本地独立模式 (Standalone，进阶场景)**：支持将本地 flac/mp3/wav 音频文件拖入小部件直接播放。
- **原生融入桌面**：通过 `Progman` 挂载实现 `Win+D` 桌面常驻不消失，配合 DirectComposition 玻璃拟态实现直通壁纸的通透质感。

---

## 2. 核心功能特性矩阵

| 功能维度 | 核心特性说明 |
| :--- | :--- |
| **视觉呈现** | 4 种舞台模式切换（全沉浸流光舞台、极简单行卡拉OK、唱片桌面胶片模式、直通壁纸玻璃模式） |
| **逐字动效** | 支持毫秒级逐字染光推进（YRC / QRC / TTML 格式），未播放字与已播放字双层色相平滑过渡 |
| **弹性动力学** | 歌词切行采用 Spring 弹簧阻尼物理动画（微位移 + 缩放 + 景深高斯散焦），消除机械跳帧 |
| **自适应色盘** | 动态提取专辑封面主色、次色与高光色，实时驱动流光背景梯度混色 |
| **系统媒体联动** | 深度接入 Windows SMTC（System Media Transport Controls），双向控制（播放/暂停/切歌/进度跳转） |
| **智能歌词源** | 本地歌词缓存 + 多源自动 fallback 检索（网易云/QQ音乐开放接口/LrcLib） |
| **桌面规范遵从** | 严格遵循 `WidgetWindow<T>` 封装、Win+D 桌面常驻、独立设置弹窗标准 |

---

## 3. 文档导航

本提案被拆分为以下专门模块文档，便于深入评估与分工开发：

- [架构与 GPUI 渲染动效管线](./architecture-and-rendering.md)
  - 详细阐述 GPUI 自定义 Element 绘制、逐字卡拉OK扫光算法、Spring 缓动曲线与流体光晕实现。
- [SMTC 系统媒体监听与歌词引擎](./smtc-and-lyrics-engine.md)
  - 阐述 Windows 媒体传输控制协议的 WinRT 异步监听、YRC/QRC 逐字解析器与时间轴高精度同步。
- [窗口常驻与系统集成规范](./window-and-integration.md)
  - 阐述 `Progman` 挂载、DirectComposition 硬件透明通道直通、设置项与 `widget-core` 的无缝装配。

---

## 4. 实施路线图 (Milestones)

### Phase 1: 歌词解析与核心排版渲染 (2 周)
- [ ] 构建 `YrcParser` / `LrcParser`，支持标准逐行与精确毫秒逐字时间戳。
- [ ] 实现基础 GPUI 歌词列表组件：当前行高亮、自动居中滚动、上下渐隐蒙版。
- [ ] 接入本地虚拟音频时间轴（模拟播放与时间同步校准）。

### Phase 2: 逐字平滑扫光与 Spring 动效管线 (2 周)
- [ ] 在 GPUI 自定义 Canvas Element 中实现双层文本遮罩（`with_mask`）逐字平滑扫光。
- [ ] 引入 Spring 物理弹簧模型，驱动歌词切换时的位移、缩放与渐隐动画。
- [ ] 实现提取封面图片色盘（K-Means 或八叉树色彩量化算法）。

### Phase 3: Windows SMTC 全局媒体捕获与网络歌词 (2 周)
- [ ] 基于 `windows` crate 实现 `GlobalSystemMediaTransportControlsSessionManager` 监听。
- [ ] 实时解析外部播放器封面、歌名、歌手、播放状态与播放进度。
- [ ] 接入多源歌词检索机制，支持本地缓存与在线匹配。

### Phase 4: 桌面级打磨与规范合入 (1 周)
- [ ] 规范化包装入 `WidgetWindow<T>` 容器，接入 `Progman` 挂载保证 `Win+D` 常驻。
- [ ] 编写符合规范的独立设置弹窗（舞台模式切换、字体大小、流光模糊强度等）。
- [ ] 执行 `cargo clippy`、`cargo fmt`、内存与 CPU 压力基准测试。
