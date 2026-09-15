# Widget Specification

<p align="center">
  <strong>English</strong> | <a href="../小组件规范.md">简体中文</a>
</p>

This specification governs the design, UI styling, and interactions of desktop widgets (plugins) in Widget-RS, ensuring all built-in and third-party widgets maintain an immersive, minimalist, and consistent user experience.

---

## 1. Core Design Principles

- **High Cohesion, Low Coupling**: Widget business logic must be fully self-contained without tight coupling to main framework states.
- **Immersive & Minimalist**: Discard traditional desktop application window chrome. Frameless design where drag-and-drop is uniformly managed by the framework.
- **Dark Theme Priority**: Follows VoltAgent deep-space terminal aesthetics. Default backgrounds should use translucent frosted glass or dark surfaces (`rgba(0x10, 0x10, 0x14, 0.95)`). Specialized widgets (like sticky notes) may offer theme palettes.
- **Frameless Experience**: Exterior borders and drag bars are handled exclusively by `WidgetWindow`. Plugins focus on core feature rendering.

---

## 2. Window Behaviors & Layout

### 2.1 Titlebars & Functional Bars
- **No Heavy System Titlebars**: Widgets must not render permanent, thick titlebars during regular use.
- **Minimalist Action Bars**: Specialized utilities (such as sticky note palette switchers or category tags) may include a compact toolbar with a height under `32px`.
- **Unified Edit Mode**: When the user enters "Edit Mode", the framework renders an emerald green (`#00d992`) drag handle above the widget. Developers only provide `drag_label` via `WidgetContent`.

### 2.2 Global Magnetic Snapping & Alignment
Every widget automatically inherits framework-level window physics without extra code:
- **Screen Edge Snapping**: Approaching within 18px of screen or multi-monitor work areas automatically snaps and aligns the window to the border.
- **Inter-Widget Snapping**: Approaching adjacent active widgets magnetically attaches edges, center lines, and bounds.
- **Desktop Pinning vs. Floating**: Widgets can reside at the desktop layer (mounted to `Progman`, never hidden by `Win+D`) or float globally.

### 2.3 Dimensions & Spacing
- **Border Radius**: Use `6px` to `8px` (`rounded(px(6.0))`).
- **Padding**: Maintain at least `8px` to `12px` between content and window boundaries.

### 2.4 Standard Settings Window Specification
Plugins providing an independent settings dialog must maintain visual coherence:
- **Creation**: Use `widget_core::default_settings_window_options(cx, initial_size)`.
- **Shell & Titlebar**: Wrap with `widget_core::render_settings_shell(title, content)` (46px standard titlebar with native dragging and close button).
- **Styling**:
  - Background: GitHub Dark (`rgb(0x0d1117)`), border (`rgb(0x30363d)`).
  - Section Headers: `widget_core::settings_section_header("...")` (12px bold, `#8b949e`).
  - Cards: `widget_core::settings_card()` (`#161b22` fill, `#30363d` border).
- **Scroll Handling**: Managed by `render_settings_shell`; do not add outer scroll containers.

### 2.5 Core Win32 Window Behaviors

#### 1. Win+D Desktop Pinning & Focus Isolation
- **Mechanism**: Normal widgets mount to `Progman` via `SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, progman)`. When `Win+D` is pressed, widgets stay anchored to the desktop wallpaper rather than minimizing.
- **Anti-Chaining**: In Win32, sibling windows owned by `Progman` form a Z-order group. Activating one widget normally brings all sibling widgets forward. We intercept `WM_WINDOWPOSCHANGING` in `plugin_wnd_proc` and append `SWP_NOZORDER` to passive activations, keeping widget focus 100% independent.

#### 2. Always-on-Top Root Cause & Solution
- **Issue 1: Widget still occluded by foreground apps**:
  - *Root Cause*: Win32 rule: *"A non-topmost window cannot own a topmost window."* Because `Progman` is a bottom-layer non-topmost window, an owned widget cannot rise above regular foreground windows.
  - *Solution*: When calling `widget_core::set_window_always_on_top(hwnd, true)`, we first unhook `Progman` (`GWLP_HWNDPARENT = 0`) before applying `HWND_TOPMOST`. Canceling unsets top-most and re-mounts to `Progman`.
- **Issue 2: Topmost toggle requires a second click to bring forward**:
  - *Root Cause*: When clicking "Always on Top" in the dashboard, the widget is not active, and `WM_WINDOWPOSCHANGING` misinterprets the activation as group chaining, appending `SWP_NOZORDER`.
  - *Solution*: We introduce an atomic `ALLOW_EXPLICIT_ZORDER` flag and invoke `BringWindowToTop(hwnd)` to immediately elevate the window.
- **Rule**: Plugins must never call `SetWindowPos` directly for Z-order; always call `widget_core::set_window_always_on_top(hwnd, is_top)`.

---

## 3. Micro-Interactions & Motion

- **Hover States**: Interactive elements must provide visual feedback (e.g. `rgba(0xffffff, 0.1)` or border highlight).
- **Smooth Animation**: Transitions and progress meters should maintain 60FPS to 120FPS fluidity.
- **No Emoji in UI**: User-facing UI elements must avoid raw emoji icons and utilize vector icons or geometric symbols.

---

## 4. Development Constraints

1. **Dead Code Elimination**: Clean up unused imports, comments, and experimental dead code.
2. **File Length Limit**: Keep any single `.rs` file under **400 lines**. Split into submodules when exceeding.
3. **MVC Architecture**:
   - `lib.rs`: Plugin registration and `Plugin` trait.
   - `model.rs`: Data models and persistence (`AppConfig`).
   - `view.rs`: UI rendering (`WidgetContent` and `Render`).
