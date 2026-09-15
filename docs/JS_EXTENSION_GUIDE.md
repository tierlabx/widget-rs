# Widget-RS JavaScript Extension Guide (GPUI Shell)

Please refer to the detailed documentation in [JS插件开发指南.md](./JS插件开发指南.md).

For quick overview:
- Extensions are located under `%APPDATA%/tierlabx/widget-rs/extensions` or `./extensions/`.
- Each extension directory contains `manifest.json` (metadata & window specs) and `main.js` (logic & declarative node render tree).
- Native GPUI rendering with zero DOM overhead.
