# Widget-RS JavaScript Extension Guide (GPUI Shell Architecture)

<p align="center">
  <strong>English</strong> | <a href="../JS插件开发指南.md">简体中文</a>
</p>

This guide is intended for developers who wish to write desktop widgets using JavaScript. `widget-rs` integrates the official **GPUI Shell** extension system from the `gpui-kit` ecosystem, allowing you to build declarative desktop widgets with lightweight JavaScript scripts (ES Modules + native QuickJS JIT environment) without needing to configure a Rust build environment.

---

## 1. Architectural Philosophy & Features

- **Native GPU Rendering, Zero DOM Overhead**: Unlike Electron or WebView solutions, GPUI Shell's JavaScript engine runs in an embedded, high-performance QuickJS JIT virtual machine. Scripts directly map declarative `View` classes and node hierarchies into GPUI GPU primitives.
- **Full JavaScript Capabilities**: Supports ES Modules, object-oriented class inheritance, computed properties, loops, conditional branches, event listeners (`on_click`), and native timers (`cx.timer.every`).
- **Inherits Core Desktop Capabilities**: Automatically inherits `widget-rs`'s Win32 click-through, `Win+D` desktop pinning, frosted glass acrylic transparency, always-on-top modes, and magnetic dragging.

---

## 2. Extension Directory Structure

Each widget extension is a self-contained directory placed inside the extensions root:

```text
extensions/
└── my_widget/
    ├── manifest.json     # Extension metadata and window options (Required)
    ├── main.js           # Business logic and View rendering entry (Required)
    ├── jsconfig.json     # Editor IDE autocompletion & type checking (Recommended)
    ├── gpui-kit.d.ts     # gpui-kit global TypeScript definition (Recommended)
    └── icon.png          # Extension icon (Optional, .png or .svg)
```

---

## 3. Configuration: `manifest.json`

Defines how the widget appears in the dashboard, its entry script, and window dimensions:

```json
{
  "id": "my_custom_widget",
  "name": "My Custom Widget",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Desktop widget built with GPUI Shell",
  "entry": "main.js",
  "icon": "clock",
  "window": {
    "width": 310.0,
    "height": 92.0,
    "transparent": true,
    "blurred": true
  }
}
```

> **Note**:
> - Entry field supports `"entry": "main.js"` or `"main": "main.js"`.
> - Setting `"blurred": true` activates Windows DirectComposition acrylic blur.

---

## 4. Script Entry: `main.js`

An extension exports a default class inheriting from `gpui-kit`'s `View`.

### Lifecycle & Core APIs

- **`init(props, cx)`**: Lifecycle hook called on creation to set initial state, register timers, or set up listeners.
- **`render(cx)`**: Returns a declarative native element tree built using `h_flex()`, `v_flex()`, `div()`, etc.
- **`cx.notify()`**: Triggers GPUI differential re-rendering when reactive state changes.
- **`cx.timer.every(interval_ms, callback)`**: Schedules recurring timer tasks.

### Complete Example: Digital Clock

```javascript
import { View, div } from "gpui-kit";
import { h_flex, v_flex } from "gpui-base";

export default class DigitalClock extends View {
  init(_props, cx) {
    this.hoursMinutes = "--:--";
    this.seconds = "00s";
    this.date = "--/--";
    this.updateTime();

    // Update time every second and trigger a re-render
    this.timer = cx.timer.every(1000, () => {
      this.updateTime();
      cx.notify();
    });
  }

  updateTime() {
    const now = new Date();
    const pad = (n) => String(n).padStart(2, "0");
    this.hoursMinutes = `${pad(now.getHours())}:${pad(now.getMinutes())}`;
    this.seconds = `${pad(now.getSeconds())}s`;

    const weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    this.date = `${pad(now.getMonth() + 1)}/${pad(now.getDate())} ${weekdays[now.getDay()]}`;
  }

  render(_cx) {
    return h_flex()
      .w_full()
      .h_full()
      .px(16)
      .py(12)
      .gap(16)
      .items_center()
      .justify_between()
      .bg("#0f172a65")
      .rounded(16)
      .border(1)
      .border_color("#ffffff18")
      .child(
        div()
          .text_size(38)
          .font_bold()
          .text_color("#f8fafc")
          .child(this.hoursMinutes)
      )
      .child(
        v_flex()
          .gap(6)
          .justify_center()
          .child(
            h_flex()
              .px(6)
              .py(2)
              .rounded(6)
              .bg("#00d99222")
              .border(1)
              .border_color("#00d99244")
              .child(
                div()
                  .text_size(11)
                  .font_bold()
                  .text_color("#00d992")
                  .child(this.seconds)
              )
          )
          .child(
            div()
              .text_size(11)
              .text_color("#94a3b8")
              .child(this.date)
          )
      );
  }
}
```

---

## 5. Chained Styling Primitives

GPUI Shell aligns closely with native GPUI fluent builder methods:

### Layout & Sizing
- `.w_full()` / `.h_full()` / `.size_full()`: Fill container bounds
- `.flex_1()`: Expand to fill available flex space
- `.px(val)` / `.py(val)` / `.p(val)`: Padding
- `.gap(val)`: Flex item gap
- `.items_center()` / `.justify_center()` / `.justify_between()`: Flex alignment

### Surface & Borders
- `.bg(hex_or_token)`: Background color (supports hex with alpha, e.g. `"#0f172a65"`)
- `.rounded(radius)`: Corner border radius in pixels
- `.border(width)`: Border thickness
- `.border_color(hex)`: Border color

### Typography
- `.text_size(size)`: Font size in pixels
- `.font_bold()` / `.font_normal()` / `.font_weight(weight)`: Font weight
- `.text_color(color)`: Text color

---

## 6. Directory Locations & Live Reloading

### Extension Search Paths
`widget-rs` automatically discovers extensions from:
1. **User Directory**: `%APPDATA%/tierlabx/widget-rs/extensions/`
2. **Local Directory**: `./extensions/` relative to the executable

### Dashboard Management
1. Open the **Dashboard** and select the **Widgets** catalog.
2. Click the **JS Extensions** filter tab at the top to display all dynamic JavaScript widgets.
3. Click **Open Extension Folder** to jump directly to the extensions directory in File Explorer.
4. After editing `main.js`, click **Refresh Extensions** to instantly hot-reload changes without restarting the app.
