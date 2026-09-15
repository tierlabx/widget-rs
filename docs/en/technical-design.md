# Widget RS - Technical Design Document

<p align="center">
  <strong>English</strong> | <a href="../技术设计.md">简体中文</a>
</p>

## 1. Technology Stack

- **Core Language**: Rust (guaranteeing memory safety and raw performance).
- **GUI Framework**: `gpui` (GPU-accelerated UI engine with component tree and reactive contexts).
- **System Tray**: `tray-icon` (cross-platform system tray and context menus).
- **Window Management**: Bridged GPUI `Window` API with low-level Win32 window handles for magnet snapping, transparency modulation, and mouse click-through.
- **Serialization & Persistence**: `serde`, `serde_json` for structured local configuration and state.
- **Dynamic Extensions**: `gpui-shell` (embedded QuickJS JIT) for JavaScript dynamic widgets.

---

## 2. Core Technical Architecture & Solutions

### 2.1 GPUI Component Architecture
- **Fluent Builder Pattern**: Views and styling are defined declaratively in Rust using GPUI's chainable methods.
- **Root Element Wrapping**: Every window root wraps its top-level view in `gpui_component::Root::new(content, window, cx)` to unify modals, sheets, tooltips, and focus graphs.

### 2.2 Multi-Window & Frameless Execution
- **Frameless Configuration**: Window options configure `titlebar: None` and `window_background: WindowBackgroundAppearance::Transparent` via `widget_core::default_widget_window_options()`.
- **Unified Container (`WidgetWindow<T>`)**: All plugin views implement `WidgetContent` and are wrapped in `WidgetWindow`. The container handles edit mode toggles, `#00d992` drag handles, border rendering, and `WS_THICKFRAME` dynamic style updates.
- **Native Window Dragging**: Invokes GPUI's `start_window_drag()` directly from the container's drag handle.

### 2.3 Win32 Desktop Persistence & Z-Order Coordination
- **Win+D Desktop Pinning (Progman Mounting)**:
  Widgets are mounted to system `Progman` via `SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, progman_hwnd)` as owned desktop windows. When a user presses `Win+D` (Show Desktop), widgets remain visible on the wallpaper.
- **Focus Isolation & Anti-Chaining**:
  In Windows, sibling windows belonging to `Progman` default to collective Z-order chaining (activating one pulls all siblings to the front). In the custom `plugin_wnd_proc` handling `WM_WINDOWPOSCHANGING`, we intercept passive activations for sibling windows and append `SWP_NOZORDER`, ensuring each widget's focus is 100% independent.
- **Always-on-Top Unhooking Mechanism**:
  Win32 strictly enforces: *"A non-topmost window cannot own a topmost window."* To make a widget always-on-top, it must first be unhooked from `Progman` via `SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, 0)` before applying `HWND_TOPMOST`. When unsetting top-most, `HWND_NOTOPMOST` is applied and the widget is re-mounted to `Progman`.
- **Explicit Z-Order Dispatch**:
  An `ALLOW_EXPLICIT_ZORDER` atomic flag prevents intentional top-most promotions from being misidentified as passive group linkages, while `BringWindowToTop(hwnd)` immediately promotes the window to the front without requiring a secondary click.

### 2.4 Magnetic Edge Snapping & Mouse Passthrough
- **Magnetic Snapping**: Window position changes are monitored. When window coordinates fall within snapping distance (e.g. 18px) of screen edges, work areas, or adjacent widgets, positions automatically snap to edges and alignment axes.
- **Mouse Click-Through**: Low-level Win32 window message hooks dynamically toggle cursor hit-testing. When enabled, mouse clicks pass through directly to underlying windows.

### 2.5 Plugin Ecosystem
- **Source-Level Plugins**: Scaffolding and registration via `widget-cli`. New plugins implement `WidgetContent` and `Plugin` traits.
- **JavaScript Dynamic Extensions**: Written in ES Modules + `View` class without requiring Rust compilation. Handled by `gpui-shell` with hot-reloading support.

### 2.6 Local Storage System
- Utilizes standard system app directories (`%APPDATA%/tierlabx/widget-rs/`).
- Separation of configuration (`config.json`) and downloaded/custom extensions (`extensions/`).

---

## 3. Project Structure

```text
widget-rs/
├── Cargo.toml                # Workspace root configuration
├── crates/                   # Core framework crates
│   ├── core/                 # widget-core: Plugin/WidgetContent traits, WidgetWindow, AppConfig
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── settings_window.rs # Unified modal shell & titlebar
│   │       └── widget_window.rs  # WidgetWindow<T> container
│   ├── app/                  # widget-rs (Application entry point)
│   │   └── src/
│   │       ├── main.rs               # Win32 subclassing & lifecycle
│   │       ├── window_manager.rs     # Multi-window handle manager & snapping
│   │       └── plugin_registry.rs    # Plugin registry (CLI-managed)
│   ├── ui/                   # widget-ui: Dashboard & shared UI components
│   │   └── src/
│   │       ├── main_window.rs        # Dashboard control center
│   │       └── components/           # Reusable UI primitives (Button, Card, Toggle)
│   └── cli/                  # widget-cli: Automation CLI for plugins & releases
├── plugins/                  # Built-in native plugin crates
│   ├── sticky/               # Sticky notes widget
│   ├── todo/                 # Task list widget
│   ├── stretchly/            # Break reminder widget
│   └── fences/               # App grid container widget
├── extensions/               # Dynamic JavaScript extensions
└── docs/                     # Documentation & design specs
    └── en/                   # English documentation
```
