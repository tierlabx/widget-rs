# Quick Start (Getting Started)

<p align="center">
  <strong>English</strong> | <a href="../快速开始.md">简体中文</a>
</p>

Welcome to Widget-RS! This project is a high-performance, extensible modern desktop widget system built with Rust and GPUI.

---

## Environment Prerequisites

Before starting, ensure your operating system meets the following requirements:

1. **Rust Toolchain**: Install the latest stable Rust via [rustup](https://rustup.rs/).
2. **C++ Build Tools**:
   - **Windows**: Install Visual Studio Build Tools with the "Desktop development with C++" workload enabled.
   - **macOS**: Run `xcode-select --install`.
   - **Linux**: Install essential packages (e.g., `build-essential`, `pkg-config`, `libx11-dev`).

---

## Clone and Run

1. **Clone the Repository**
   ```bash
   git clone https://github.com/tierlabx/widget-rs.git
   cd widget-rs
   ```

2. **Launch Application**
   ```bash
   cargo run --release
   ```
   *Tip: Running with `--release` enables GPU optimizations, smooth 120Hz rendering, and minimal memory footprints.*

---

## Using Widgets

- **Dashboard**: Upon startup, the management dashboard opens, allowing you to configure widgets, toggles, and global settings.
- **Edit Mode**: In the system tray context menu, select **"Edit Layout"** (or toggle it in the Dashboard). A green drag handle (`#00d992`) appears at the top of each widget, allowing you to freely drag, reposition, and magnetically snap widgets.
- **Adding Plugins**: Check out the [Plugin Development Guide](plugin-development-guide.md) to learn how to install and uninstall native widgets via `widget-cli`.
- **Dynamic JavaScript Widgets**: Check out the [JavaScript Plugin Guide](js-plugin-guide.md) to create lightweight widgets without a Rust compiler.

---

## Next Steps

- Read [Architecture Design](architecture.md) to understand internal systems and lifecycle.
- Read [Widget Specification](widget-spec.md) to learn design rules and Win32 desktop integration principles.
- Read [FAQ](faq.md) for troubleshooting common questions.
