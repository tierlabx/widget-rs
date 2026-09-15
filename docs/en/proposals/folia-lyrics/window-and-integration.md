# Folia Lyrics: Window Hosting & System Integration Specification

<p align="center">
  <strong>English</strong> | <a href="../../../proposals/folia-lyrics/window-and-integration.md">简体中文</a>
</p>

This document specifies how the widget adheres to `widget-rs` architectural standards, including `WidgetWindow<T>` encapsulation, `Progman` desktop residency on `Win+D`, settings modal standards, and source module layout constraints.

---

## 1. Core Window Capabilities & Host Integration

Per project rules, widgets must never exist as unmanaged free windows and must adhere to three mandatory constraints:

### 1.1 `WidgetWindow<T>` Container Wrapping
- The widget content struct must implement `widget_core::WidgetContent`:
  ```rust
  impl WidgetContent for LyricsWidgetContent {
      fn plugin_id(&self) -> &'static str {
          "folia-lyrics"
      }
      fn drag_label(&self) -> String {
          "Folia Lyrics".to_string()
      }
      fn show_drag_handle(&self) -> bool {
          // Hide drag handle in full-screen mode
          !self.is_fullscreen
      }
  }
  ```
- **Strict Prohibition**: Never manually draw edit borders, drag handles, or edit-mode checks in widget views. All window chrome is driven centrally by `WidgetWindow`.

### 1.2 Win+D Desktop Persistence (`Progman` Pinning)
- Designed to coexist with the desktop wallpaper: when the user presses `Win+D` (Show Desktop), regular apps minimize, but the widget **must remain visible on the desktop**.
- Implementation: When calling `spawn_window`, ensure Win32 `SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, progman_hwnd)` attaches the window to `Progman`, preserving `DwmExtendFrameIntoClientArea` for direct wallpaper transparency.

### 1.3 Always-on-Top & Z-Order Management
- Toggle top-most via `widget_core::set_window_always_on_top(hwnd, is_top)`. Under the hood, this uses `HWND_TOPMOST` and `HWND_NOTOPMOST` (never `HWND_BOTTOM`).

---

## 2. Dedicated Settings Window Specification

When the user opens settings from the dashboard or tray context menu, the modal implementation must strictly comply with `widget-core`:

```rust
fn build_settings_window(&self, cx: &mut App) -> WindowHandle<SettingsWindow> {
    let initial_size = size(px(480.0), px(560.0));
    let options = widget_core::default_settings_window_options(cx, initial_size);
    cx.open_window(options, |cx| {
        cx.new(|_| SettingsWindow::new())
    })
}
```

- **Outer Shell**: Wrapped with `widget_core::render_settings_shell("Lyrics Settings", content)`. Never manually build titlebar dragging, close buttons, or outer scroll containers.
- **Sections & Cards**: Form controls use `widget_core::settings_section_header("Display Modes")` and `widget_core::settings_card()` for global visual harmony.
- **No Emoji**: UI elements must avoid raw emoji icons.

---

## 3. Configuration Persistence Schema

Stored locally at `%APPDATA%/tierlabx/widget-rs/plugins/folia-lyrics/config.json`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoliaLyricsConfig {
    /// Visual mode: TypographyPv(Full-screen) / FlowStage / MinimalSingleLine / VinylRecord
    pub visual_mode: VisualMode,
    /// Enable kinetic Typography PV bounce & drift
    pub enable_pv_animation: bool,
    /// Lyric font size (px)
    pub lyric_font_size: f32,
    /// Primary font family
    pub font_family: String,
    /// Ambient light stage intensity (0.0 ~ 1.0)
    pub flow_stage_intensity: f32,
    /// Gaussian blur radius (px)
    pub backdrop_blur_radius: f32,
    /// Enable millisecond word-by-word sweeps
    pub enable_word_by_word: bool,
    /// Show translated lyrics
    pub show_translation: bool,
    /// Pin to desktop Progman (Win+D persistence)
    pub pin_to_desktop: bool,
    /// Whether in full-screen immersion mode
    pub fullscreen_mode: bool,
}
```

---

## 4. Module Decomposition (<400 Lines per File Rule)

When implementing under `plugins/folia-lyrics/src/`, files must remain strictly under 400 lines:

```text
plugins/folia-lyrics/src/
├── lib.rs                 # Plugin lifecycle & Plugin trait (< 160 lines)
├── types.rs               # Config, lyric models, PV modes & events (< 200 lines)
├── engine/                # Background service submodule
│   ├── mod.rs             # Engine facade & event dispatching (< 150 lines)
│   ├── smtc.rs            # WinRT SMTC session capture & extrapolation (< 300 lines)
│   ├── parser.rs          # YRC/LRC parser (< 260 lines)
│   ├── pv_animator.rs     # [Typography PV Core] Spring models & word physics (< 250 lines)
│   └── fetcher.rs         # Online retrieval & cache management (< 240 lines)
├── ui/                    # GPUI Presentation submodule
│   ├── mod.rs             # View composition & WidgetContent (< 260 lines)
│   ├── stage.rs           # Fluid stage canvas drawing (< 280 lines)
│   ├── typography_pv.rs   # [Typography PV Core] Kinetic word bounce Element (< 380 lines)
│   └── controls.rs        # HUD overlay & media buttons (< 220 lines)
└── settings/              # Settings submodule
    ├── mod.rs             # Submodule export (< 20 lines)
    └── view.rs            # Standard settings shell (< 280 lines)
```
