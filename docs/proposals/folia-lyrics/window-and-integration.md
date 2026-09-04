# Folia 歌词：窗口宿主与系统集成规范

本文档阐述小部件如何严格遵守 `widget-rs` 架构约束，包括 `WidgetWindow<T>` 封装、Win+D 桌面常驻、设置弹窗规范以及工程模块拆分。

---

## 1. 核心窗口能力与宿主集成规范

根据 `widget-rs` 项目核心规则，小部件严禁作为脱离框架的自由窗口存在，必须遵守以下三项强约束：

### 1.1 `WidgetWindow<T>` 容器包装
- 小部件内容结构体必须实现 `widget_core::WidgetContent` trait：
  ```rust
  impl WidgetContent for LyricsWidgetContent {
      fn plugin_id(&self) -> &'static str {
          "folia-lyrics"
      }
      fn drag_label(&self) -> String {
          "桌面流光歌词".to_string()
      }
      // 允许根据展示模式选择是否隐藏默认拖拽手柄
      fn show_drag_handle(&self) -> bool {
          true
      }
  }
  ```
- 严禁在小组件的视图中手动绘制编辑模式边框、拖拽条或编辑模式检测；所有外围容器行为均由 `WidgetWindow` 统一驱动。

### 1.2 Win+D 桌面常驻特性 (Progman 绑定)
- 小部件定位为与桌面壁纸共生，当用户按下键盘 `Win+D`（显示桌面）时，普通应用被最小化，小组件**必须始终保留在桌面之上**。
- 实现要求：在 `spawn_window` 建立时调用统一的系统窗口样式配置，必须确保 Win32 层调用 `SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, progman_hwnd)` 完成与系统桌面的挂载，且维持 `DwmExtendFrameIntoClientArea` 确保 DirectComposition 透明通道直通壁纸。

### 1.3 置顶与窗口分级控制
- 遵循项目规则：置顶统一调用 `widget_core::set_window_always_on_top(hwnd, is_top)`，底层使用 `HWND_TOPMOST` 与 `HWND_NOTOPMOST`，严禁使用 `HWND_BOTTOM`。

---

## 2. 独立设置弹窗规范 (Settings Window)

当用户在控制面板或右键菜单中点击“插件设置”时，弹窗实现必须 100% 遵从 `widget-core` 规范：

```rust
// 伪代码参考
fn build_settings_window(&self, cx: &mut App) -> WindowHandle<SettingsWindow> {
    let initial_size = size(px(480.0), px(560.0));
    let options = widget_core::default_settings_window_options(cx, initial_size);
    cx.open_window(options, |cx| {
        cx.new(|_| SettingsWindow::new())
    })
}
```

- **外层容器**：必须使用 `widget_core::render_settings_shell("流光歌词设置", content)` 包装，严禁手写拖拽栏、关闭按钮或外层滚动。
- **区块与表单**：必须使用 `widget_core::settings_section_header("显示模式")` 与 `widget_core::settings_card()` 构建设置组，保证全局设置一致性。
- **禁止使用 Emoji**：设置界面的所有标签、提示文字与说明中严禁包含 emoji。

---

## 3. 配置持久化 Schema (Config Model)

配置文件统一保存在系统应用数据目录 `plugins/folia-lyrics/config.json`：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoliaLyricsConfig {
    /// 舞台视觉模式: ImmersiveStage(全流光) / MinimalSingleLine(极简单行) / VinylRecord(黑胶) / DesktopPure(纯净桌面)
    pub visual_mode: VisualMode,
    /// 歌词字号大小 (px)
    pub lyric_font_size: f32,
    /// 歌词主字体名称（默认系统苹方/思源黑体/Segoe UI）
    pub font_family: String,
    /// 背景流光强度 (0.0 ~ 1.0)
    pub flow_stage_intensity: f32,
    /// 高斯模糊半径 (px)
    pub backdrop_blur_radius: f32,
    /// 是否开启逐字毫秒扫光
    pub enable_word_by_word: bool,
    /// 是否显示外语翻译
    pub show_translation: bool,
    /// 桌面是否常驻置底 (True = 贴合桌面 Progman, False = 普通窗口浮动)
    pub pin_to_desktop: bool,
}
```

---

## 4. 插件工程模块拆分（单文件 ≤ 400 行红线）

正式在 `plugins/folia-lyrics/src/` 中创建工程时，必须严格遵守单文件行数限制，结构划分如下：

```text
plugins/folia-lyrics/src/
├── lib.rs                 # 插件生命周期入口、Plugin trait 实现（< 150 行）
├── types.rs               # 配置结构、歌词模型与事件类型定义（< 200 行）
├── engine/                # 后台服务子模块
│   ├── mod.rs             # 引擎对外门面与状态分发（< 150 行）
│   ├── smtc.rs            # Windows WinRT SMTC 会话捕获器（< 300 行）
│   ├── parser.rs          # YRC/LRC 逐字歌词文本解析器（< 280 行）
│   └── fetcher.rs         # 在线歌词检索与本地缓存管理（< 260 行）
├── ui/                    # 前端 GPUI 渲染子模块
│   ├── mod.rs             # 视图装配与 WidgetContent trait 实现（< 220 行）
│   ├── stage.rs           # 动态流光舞台背景 Canvas 绘制（< 300 行）
│   ├── karaoke.rs         # 逐字文本度量、遮罩与平滑扫光 Element（< 350 行）
│   └── controls.rs        # 曲目浮标与媒体控制按钮组件（< 250 行）
└── settings/              # 设置界面子模块
    └── view.rs            # 基于 settings_shell 的标准设置界面（< 280 行）
```

通过这一明确清晰的子模块划分，每一项业务职责完全解耦，并且每个代码文件均在 350 行以内，严格满足项目的架构和工程规范。
