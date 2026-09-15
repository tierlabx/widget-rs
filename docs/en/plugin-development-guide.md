# Widget-RS Plugin Development Guide

<p align="center">
  <strong>English</strong> | <a href="../插件开发指南.md">简体中文</a>
</p>

Widget-RS features a high-performance, developer-friendly source-level plugin architecture. This guide walks you through building your own desktop widget from scratch and integrating it seamlessly using `widget-cli`.

---

## 1. Plugin Architecture Overview

Widget-RS completely decouples the **Core Framework (`widget-core`)** from **Feature Plugins (`plugins/*`)**:
- **Rendering Engine**: Native GPUI rendering for GPU acceleration, 120Hz refresh rate support, and minimal memory footprints.
- **Unified Window Container**: `WidgetWindow<T>` encapsulates common desktop window capabilities (edit mode toggles, `#00d992` drag handles, border highlights, mouse click-through, always-on-top, and Progman pinning). Plugin authors only need to write their content views.
- **Dynamic Registry**: With `plugin_registry.rs` and the `widget-cli` tool, plugins can be registered and removed with one command without editing the main application code by hand.

---

## 2. Core Concepts

### 2.1 The `WidgetContent` Trait

`WidgetContent` is the minimal interface a widget view must implement:

```rust
pub trait WidgetContent: Render + Sized + 'static {
    /// Returns the unique plugin ID (must match Plugin::id)
    fn plugin_id(&self) -> &'static str;

    /// Label displayed on the drag handle during Edit Mode
    fn drag_label(&self) -> &'static str { "Drag to move" }

    /// Whether to render the edit mode drag handle (default: true)
    fn show_drag_handle(&self) -> bool { true }
}
```

### 2.2 The `WidgetWindow` Container

Wrapping your view inside `WidgetWindow<T: WidgetContent>` automatically provides:
- Edit mode detection and style toggling
- A unified emerald green drag handle (`#00d992`) triggering native window drag
- Resizable border highlights during edit mode

You **never** need to re-implement window drag or border logic in your plugin.

---

## 3. Creating a Plugin Step-by-Step

### 3.1 Create a Library Crate
```bash
cargo new --lib plugins/my_clock
```

### 3.2 Configure Dependencies
In `plugins/my_clock/Cargo.toml`:
```toml
[package]
name = "my_clock"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui.workspace = true
gpui-component.workspace = true
widget-core.workspace = true
```

### 3.3 Implement the Widget & Plugin
In `plugins/my_clock/src/lib.rs`:

```rust
use gpui::*;
use widget_core::Plugin;

// 1. Define Widget View State
struct ClockWidget {
    // Custom widget state
}

impl ClockWidget {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

// 2. Implement WidgetContent
impl widget_core::WidgetContent for ClockWidget {
    fn plugin_id(&self) -> &'static str { "my_clock" }
    fn drag_label(&self) -> &'static str { "Drag Clock" }
}

// 3. Implement Render — focus exclusively on your UI
impl Render for ClockWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .justify_center()
            .items_center()
            .size_full()
            .bg(rgba(0x050507d9))
            .text_xl()
            .text_color(rgb(0xFFFFFF))
            .child("Hello, Widget!")
    }
}

// 4. Define Plugin Entry
pub struct ClockPlugin;

impl Plugin for ClockPlugin {
    fn id(&self) -> &'static str { "my_clock" }
    fn name(&self) -> &'static str { "My Clock" }
    fn description(&self) -> &'static str { "A desktop clock widget" }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = widget_core::default_widget_window_options(
            cx, "my_clock", (100.0, 100.0, 200.0, 100.0)
        );

        cx.open_window(options, |window, cx| {
            let content = cx.new(|cx| ClockWidget::new(window, cx));
            let widget_window = cx.new(|_cx| widget_core::WidgetWindow::new(content));
            cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
        })
        .unwrap()
        .into()
    }
}

// 5. Export Factory Function
pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(ClockPlugin)
}
```

---

## 4. Don't Reinvent the Wheel

The following behaviors are centrally handled by the framework—**do not duplicate them in your plugin**:

| Feature | Managed By |
| :--- | :--- |
| Edit Mode Detection (`UIState::is_edit_mode`) | `WidgetWindow` |
| Window Style Switching (`update_window_edit_mode`) | `WidgetWindow` |
| Edit Mode Drag Handle | `WidgetWindow` |
| Edit Mode Border Highlight | `WidgetWindow` |
| Window Bounds Recovery (`resolve_plugin_bounds`) | `default_widget_window_options` |
| Window Options Boilerplate | `default_widget_window_options` |

---

## 5. Advanced: Conditional Drag Handles

If your widget needs to conditionally suppress the drag handle (e.g. in full-screen or expanded mode), simply override `show_drag_handle`:

```rust
impl widget_core::WidgetContent for MyWidget {
    fn plugin_id(&self) -> &'static str { "my_widget" }

    fn show_drag_handle(&self) -> bool {
        !self.is_expanded
    }
}
```

---

## 6. Advanced: Dedicated Settings Window

If your plugin provides independent settings (opened via the gear icon on the dashboard card), follow the unified modal guidelines:

1. **Implement `Plugin::build_settings_window`**:
   ```rust
   impl Plugin for MyPlugin {
       fn build_settings_window(&self, cx: &mut App) {
           let options = widget_core::default_settings_window_options(cx, (420.0, 560.0));
           cx.open_window(options, |window, cx| {
               let view = cx.new(|cx| MySettingsView::new(window, cx));
               cx.new(|cx| gpui_component::Root::new(view, window, cx))
           })
           .unwrap();
       }
   }
   ```

2. **Wrap in `render_settings_shell`**:
   ```rust
   use widget_core::{render_settings_shell, settings_card, settings_section_header};

   impl Render for MySettingsView {
       fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
           let content = div()
               .flex()
               .flex_col()
               .p(px(16.0))
               .gap(px(12.0))
               .child(settings_section_header("General Settings"))
               .child(
                   settings_card()
                       .p(px(14.0))
                       .child("Config content here..."),
               );

           render_settings_shell("My Plugin - Settings", content)
       }
   }
   ```

---

## 7. Installing Plugins with CLI

Run `widget-cli` from the workspace root:
```bash
cargo run -p widget-cli -- plugin add my_clock --path plugins/my_clock
```
This automatically updates `crates/app/Cargo.toml` and registers `my_clock::create_plugin()` in `plugin_registry.rs`.

Now launch the app:
```bash
cargo run
```

---

## 8. Data Persistence

Persist widget configuration using `AppConfig`:
```rust
// Save data
cx.update_global::<AppConfig, _>(|cfg, _| {
    cfg.set_plugin_data("my_clock", &my_data); // my_data must implement serde::Serialize
});
widget_core::save_config_now(cx); // Flush immediately

// Load data
let saved_data: MyData = cx
    .try_global::<AppConfig>()
    .and_then(|cfg| cfg.get_plugin_data::<MyData>("my_clock"))
    .unwrap_or_default();
```

---

## 9. Uninstalling Plugins

To remove the plugin:
```bash
cargo run -p widget-cli -- plugin remove my_clock
```
