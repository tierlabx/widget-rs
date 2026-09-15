# Widget-RS JavaScript Extension Guide (GPUI Shell)

<p align="center">
  <strong>English</strong> | <a href="./JS插件开发指南.md">简体中文</a>
</p>

Please refer to the comprehensive guides:
- **English**: [JavaScript Extension Guide](./en/js-plugin-guide.md)
- **简体中文**: [JS插件开发指南](./JS插件开发指南.md)

---

### Quick Overview
- **Extension Location**: `%APPDATA%/tierlabx/widget-rs/extensions` or `./extensions/`.
- **Extension Structure**: Each directory contains `manifest.json` (metadata & window specs) and `main.js` (business logic & declarative element tree).
- **Zero DOM Overhead**: Direct GPU rendering via embedded QuickJS JIT into GPUI.
- **Desktop Integration**: Inherits native DirectComposition acrylic blur, `Progman` desktop pinning, and drag-and-drop.
