# Widget RS Desktop System - Product Requirements Document (PRD)

<p align="center">
  <strong>English</strong> | <a href="../产品需求文档.md">简体中文</a>
</p>

## 1. Product Overview

Widget-RS is a modern desktop widget system built with native GPU rendering (GPUI), adhering to VoltAgent's deep-space terminal aesthetics. It comprises a centralized management dashboard and independent floating desktop widgets (such as Sticky Notes and Todo Lists). The platform delivers an ultra-fast, visually striking, and focused efficiency toolset for developers and power users, backed by an extensible plugin ecosystem.

---

## 2. Target Users

- Developers, software engineers, and productivity-focused enthusiasts.
- Users who frequently record notes and manage tasks while favoring terminal-like, dark-mode desktop aesthetics.
- Programmers who wish to extend desktop functionality by writing custom native Rust or JavaScript widgets.

---

## 3. Design Language & Visual Aesthetics

Adheres strictly to the **VoltAgent Design System**, creating a high-performance command terminal atmosphere:
- **Core Palette**: Abyss Black (`#050507`), Carbon Surface (`#101010`), Emerald Signal Green (`#00d992`), and Warm Charcoal Border (`#3d3a39`).
- **Typography Stack**: `system-ui` (Headings), `Inter` (Body & UI), and `SFMono-Regular` (Code).
- **Visual Depth**: Constructed via 1px–2px borders, green accent glows, and subtle backdrop shadows.
- **Animations**: Fluid GPU-driven transitions for hovers, window toggles, and state changes.

---

## 4. Core Functional Modules

### 4.1 Management Dashboard (Main Window)
The central command center:
- **Navigation Sidebar**: Switch between Dashboard, Widget Catalog, Settings, and JS Extensions.
- **Content Area**: Detailed widget configuration, toggles, positioning, and extension management.

### 4.2 Sticky Note Widget
- **Window Form**: Lightweight frameless floating window (default 320x360), 8px border radius.
- **Features**: Fast note taking, Markdown formatting, and selectable theme color cards.

### 4.3 Todo List Widget
- **Window Form**: Frameless floating window (default 360x400), 8px border radius.
- **Features**: Task creation, inline completion toggles, deletion, and persistent state.

### 4.4 System-Level Window Interactions
- **Always-on-Top**: Configurable per-widget to prevent occlusion by other application windows.
- **Magnetic Edge Snapping**: Real-time magnetic attraction when dragging near multi-monitor edges and adjacent widgets.
- **Mouse Click-Through**: Passes cursor clicks through the widget to underlying desktop apps.
- **System Tray**: Persistent tray icon with context menu for quick toggle of edit mode, widgets, and clean shutdown.

### 4.5 Plugin Ecosystem
- **Native Rust Plugins**: Standardized `Plugin` and `WidgetContent` interfaces via `widget-cli`.
- **Dynamic JavaScript Extensions**: DOM-free QuickJS JIT execution via `gpui-shell` with live hot-reloading.

---

## 5. Data Storage & Privacy

- **100% Local Storage**: Zero cloud dependencies. All user data, widget positions, and extension configurations are stored locally on the client machine under `%APPDATA%/tierlabx/widget-rs/`, ensuring privacy, security, and instantaneous response.
