# Contributing to Widget-RS

<p align="center">
  <strong>English</strong> | <a href="CONTRIBUTING.md">简体中文</a>
</p>

Thank you for considering contributing to Widget-RS! This project aims to build a modern, high-performance desktop widget platform with Rust and GPUI. Whether you are reporting bugs, optimizing code, developing new widgets, or improving documentation, all contributions are welcome!

---

## 👨‍💻 Development Setup

1. **Environment**: Ensure you have the latest stable [Rust](https://rustup.rs/) (>= 1.75) installed.
2. **Formatting & Linting**: Code style and clippy rules are strictly configured via `rustfmt.toml` and `clippy.toml`. Before submitting code, always run the following checks from the workspace root:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   ```

---

## 🛠 Commit Conventions (Conventional Commits)

To cooperate with GitHub Actions and the `widget-cli release` automated version release tool, this project **strictly enforces Conventional Commits**.

The commit message format must follow:
```text
<type>(<scope>): <subject>
```

- **type** must be one of the following keywords:
  - `feat`: A new feature (triggers minor version bump and release note entry)
  - `fix`: A bug fix (triggers patch version bump and release note entry)
  - `docs`: Documentation-only changes (e.g., README.md)
  - `style`: Code style/formatting changes that do not affect logic
  - `refactor`: Code refactoring (neither a new feature nor a bug fix)
  - `perf`: Performance improvements
  - `test`: Adding or updating tests
  - `chore`: Changes to build process, tooling, or dependencies
- **scope** (optional): The affected module, e.g., `ui`, `core`, `sticky`, `todo`, `cli`.
- **subject**: A concise description of the change in imperative mood.

> ✅ Good example: `feat(sticky): add color picker support`  
> ❌ Bad example: `fixed some bugs` or `update widget`

---

## 🧩 Developing a New Plugin

Widget-RS uses a decoupled plugin architecture. To maintain long-term code quality, we **strictly require UI & Logic Separation (MVC pattern)**.

### Development Steps

1. Create a new Cargo library package under `plugins/`:
   ```bash
   cargo new --lib plugins/my_plugin
   ```
2. Organize your code structure:
   - `src/model.rs`: Pure data structures, state management, and persistence logic.
   - `src/view.rs`: GPUI rendering implementation (implements `Render` and `WidgetContent` traits).
   - `src/lib.rs`: Plugin entry point implementing `widget_core::Plugin`.
3. Your UI view struct must implement `widget_core::WidgetContent`, providing `plugin_id()` and `drag_label()`. Window-level behaviors (edit mode, drag handle, borders) are automatically handled by `WidgetWindow`—**do not re-implement them in your plugin**.
4. Implement `widget_core::Plugin` in `lib.rs`, wrapping your view in `WidgetWindow`:
   ```rust
   use gpui::*;
   use widget_core::{Plugin, WidgetWindow, default_widget_window_options};

   pub struct MyPlugin;

   impl Plugin for MyPlugin {
       fn id(&self) -> &'static str { "my_plugin" }
       fn name(&self) -> &'static str { "My Custom Widget" }
       fn description(&self) -> &'static str { "A custom desktop widget" }

       fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
           let options = default_widget_window_options(cx, "my_plugin", (100.0, 100.0, 300.0, 300.0));
           cx.open_window(options, |window, cx| {
               let content = cx.new(|cx| MyWidget::new(window, cx));
               let widget_window = cx.new(|_cx| WidgetWindow::new(content));
               cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
           }).unwrap().into()
       }
   }
   ```
5. Export the plugin factory entry point:
   ```rust
   pub fn create_plugin() -> std::sync::Arc<dyn widget_core::Plugin> {
       std::sync::Arc::new(MyPlugin)
   }
   ```
6. Register the plugin automatically via `widget-cli`:
   ```bash
   cargo run -p widget-cli -- plugin add my_plugin --path plugins/my_plugin
   ```

> **Note**: Window drag handles, edit mode borders, Z-order handling, and window style updates are handled uniformly by the `WidgetWindow` container.  
> For detailed API guides, see [Plugin Development Guide](docs/en/plugin-development-guide.md) and [Widget Specification](docs/en/widget-spec.md).

---

## 📦 Packaging & Building

This project uses [cargo-packager](https://github.com/tauri-apps/cargo-packager) for application packaging and installer distribution.

To build an installer package locally:
```bash
cargo packager --release
```
*(Note: Run `cargo install cargo-packager --locked` if you have not installed the packager tool yet.)*

Looking forward to your pull requests! 🚀
