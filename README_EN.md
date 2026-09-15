# Widget-RS 🎨

<p align="center">
  <strong>English</strong> | <a href="README.md">简体中文</a>
</p>

[![CI](https://github.com/tierlabx/widget-rs/actions/workflows/packager.yml/badge.svg)](https://github.com/tierlabx/widget-rs/actions/workflows/packager.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight, high-performance desktop widget system built with **Rust + GPUI**. Features always-on-top mode, mouse click-through, native multi-screen edge snapping, system tray integration, and an extensible dual-track plugin ecosystem.

<p align="center">
  <img src="assets/screenshot.png" alt="Widget-RS Preview" width="100%">
</p>

---

## 🌟 Key Features

- **⚡ Blazing Performance & Modern GPU Rendering**
  - Powered by **Rust + GPUI** GPU-accelerated rendering engine with millisecond cold-start response and minimal CPU/GPU overhead (base resident memory ~30MB–50MB).
  - Native DirectComposition hardware transparency pipeline providing seamless wallpaper passthrough without white border artifacts or window flicker.

- **🪟 Deep Native Desktop Integration**
  - **Win+D Desktop Persistence**: Deeply mounted to system `Progman`. Pressing `Win+D` (Show Desktop) preserves widgets alongside desktop wallpaper instead of hiding or minimizing them.
  - **Isolated Focus & Anti-Chaining**: Win32 message interception eliminates passive Z-order linkage; focusing one widget will never inadvertently pull other sibling widgets to the foreground.
  - **Dual Z-Order Modes & Click-Through**: Seamlessly switch between Desktop Wallpaper Layer and Global Always-on-Top (`HWND_TOPMOST`), with optional mouse click-through for zero workflow interference.

- **🧲 Smooth Magnet Snapping & Unified Layout**
  - **Multi-Monitor Edge & Inter-Widget Snapping**: Dragging widgets near screen borders or adjacent widgets triggers automatic magnetic snapping with edge, center, and alignment detection.
  - **One-Click Edit Mode**: Framework-level unified drag handle, border highlights, and position controls. Layout changes persist automatically and restore accurately upon launch.

- **🧩 Cohesive Dual-Track Plugin Ecosystem**
  - Unified `WidgetWindow<T>` container architecture: plugins only implement the `WidgetContent` business logic without dealing with drag handles, edit mode, or window boilerplate.
  - **Rich Built-in Widgets**: Sticky Notes (`Sticky`), Minimalist Tasks (`Todo`), Break Reminder (`Stretchly`), and Desktop App Grid (`Fences`).
  - **Dynamic JavaScript Widgets (GPUI Shell)**: Write full desktop widgets in standard ES Modules + `View` class without requiring a Rust toolchain. Zero DOM overhead, direct GPU pipeline mapping, live hot-reloading, and centralized dashboard control.
  - **Automated CLI Scaffolding**: Built-in `widget-cli` command-line tool for source-level one-click creation, registration, building, and management of plugins.

- **🎛️ Modern Control Center & Standardized Settings**
  - Modern dark-themed dashboard for managing plugin toggles, auto-start, global edit mode, and tray resident settings.
  - Standardized settings modal (`render_settings_shell`) delivering a unified visual language and smooth interactions across all plugins.

---

## 🏗 Architecture Overview

Widget-RS utilizes a layered, decoupled architecture to balance core stability with plugin extensibility:

<p align="center">
  <img src="assets/architecture.svg" alt="Widget-RS Architecture" width="100%">
</p>

For comprehensive architecture details, see [Architecture Documentation](docs/en/architecture.md).

---

## 🚀 Quick Start

### 1. Prerequisites
Ensure you have a recent version of [Rust](https://rustup.rs/) installed (1.75+ recommended).

### 2. Build & Run
```bash
git clone https://github.com/tierlabx/widget-rs.git
cd widget-rs

# Run debug build
cargo run

# Build release package
cargo packager --release
```

### 3. Usage Guide
- **Edit Mode**: Right-click the system tray icon, select **Dashboard**, and toggle Edit Mode to drag widgets and experience edge snapping.
- **Config Persistence**: Widget positions, sizes, and active states persist automatically to local configuration files.

For more details, check out the [Quick Start Guide](docs/en/quick-start.md).

---

## 🧩 Plugin Management (CLI)

Widget-RS includes `widget-cli`, a dedicated command-line tool for automated source-level plugin installation, uninstallation, and release management.

### Install & Uninstall Native Plugins
```bash
# Add a local plugin (automatically injects dependencies into Cargo.toml & registers source)
cargo run -p widget-cli -- plugin add <plugin_name> --path <local_path>
# Example:
cargo run -p widget-cli -- plugin add my_clock --path ../plugins/my_clock

# Uninstall an installed plugin
cargo run -p widget-cli -- plugin remove <plugin_name>
# Example:
cargo run -p widget-cli -- plugin remove sticky_plugin
```

### Developing Your Own Plugin
Creating a new native plugin is simple:
1. Create a new Rust library crate (`cargo new --lib plugins/my_plugin`)
2. Add the `widget-core` dependency.
3. Follow the **UI & Logic Separation** rule: `model.rs` for state/persistence, `view.rs` for GPUI rendering.
4. Implement `widget_core::WidgetContent` in `view.rs` (only specify `plugin_id` and `drag_label`); window-level capabilities (edit mode, dragging, borders) are automatically handled by `WidgetWindow`.
5. Implement `widget_core::Plugin` in `lib.rs` and wrap your view in `WidgetWindow`:
```rust
fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
    let options = widget_core::default_widget_window_options(cx, "my_plugin", (100.0, 100.0, 300.0, 300.0));
    cx.open_window(options, |window, cx| {
        let content = cx.new(|cx| MyWidget::new(window, cx));
        let widget_window = cx.new(|_cx| widget_core::WidgetWindow::new(content));
        cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
    }).unwrap().into()
}
```
6. Register it using CLI and build!

> For full API specifications and examples, see the [Plugin Development Guide](docs/en/plugin-development-guide.md).  
> For JavaScript dynamic extensions, see the [JavaScript Plugin Guide](docs/en/js-plugin-guide.md).

---

## 🔖 Releases & Versioning

Widget-RS features automated version management driven by [Conventional Commits](https://www.conventionalcommits.org/). It calculates semantic versions, updates `Cargo.toml`, generates `CHANGELOG.md`, and pushes Git tags automatically:

```bash
# Calculate version from commit history and release
cargo run -p widget-cli -- release

# Dry-run preview of next version and CHANGELOG
cargo run -p widget-cli -- release --dry-run

# Manually specify a release version
cargo run -p widget-cli -- release --version 0.2.0
```

Pushing a version tag triggers GitHub Actions to automatically build NSIS and WiX MSI installers and publish a GitHub Release.

---

## 🛠 Tech Stack

| Domain | Technology | Description |
| :--- | :--- | :--- |
| **Rendering Engine** | [GPUI](https://gpui.rs/) | GPU-accelerated modern Rust UI framework |
| **UI Components** | gpui-component | Standard reusable UI control primitives |
| **Window Subsystem** | windows-sys 0.52 | Native Win32 message hooks and window styles |
| **System Tray** | tray-icon | Cross-platform system tray and context menu support |
| **Automation & Tooling** | widget-cli, GitHub Actions | Versioning CLI, CI/CD automated packaging |

---

## 🤝 Contributing

Contributions are warmly welcomed! Whether you want to fix bugs, create new widgets, or improve documentation, please read our [Contributing Guide](CONTRIBUTING_EN.md) for workflow details and coding standards.

---

## 💖 Acknowledgements

Widget-RS is made possible thanks to inspiring open-source foundations:

- [Zed Industries / GPUI](https://github.com/zed-industries/zed) — Ultra-fast, elegant GPU-driven UI engine.
- [gpui-component](https://github.com/longbridge/gpui-component) — High quality GPUI component primitives.
- [Tauri Team](https://github.com/tauri-apps) — Excellent cross-platform foundation (`tao` & `tray-icon`).
- [Folia-Major](https://github.com/chthollyphile/folia-major) — Immersive dynamic visuals and music typography inspiration.
- All contributors, issue reporters, and community members!

---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
