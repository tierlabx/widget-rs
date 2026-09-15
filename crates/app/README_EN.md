# Widget RS App Crate

<p align="center">
  <strong>English</strong> | <a href="README.md">简体中文</a>
</p>

`widget-app` is the core application backbone module of `widget-rs`, responsible for the lifecycle and OS integration of the entire desktop platform.

## Key Responsibilities

- **Entry Point & Lifecycle Management**: Implements `main`, initializes the GPUI application context and runtime environment.
- **Configuration & Persistence** (`config`): Loads, updates, and serializes global application settings (`AppConfig`).
- **Window Management** (`window`): Encapsulates desktop-grade window control, bridging native GPUI window handles with low-level Win32 APIs for always-on-top modes, mouse click-through, and desktop Z-order anchoring.
- **Plugin Orchestration** (`plugin`): Centrally coordinates native and dynamic widget plugins, handling registration, discovery, and active states.
- **System Integration** (`system` & `tray`): Provides system tray icon management, context menus, and registry-based auto-start upon boot.
- **Asset Mounting** (`assets`): Injects bundled application resources (such as icons and brand assets).

## Architectural Boundary

This crate exclusively handles framework orchestration and low-level OS environments (e.g., Windows APIs). It contains no direct UI layout or view rendering implementations, completely delegating all UI presentation to `widget-ui` and individual plugins.
