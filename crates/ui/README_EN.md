# Widget RS UI Crate

<p align="center">
  <strong>English</strong> | <a href="README.md">简体中文</a>
</p>

`widget-ui` encompasses the core presentation and interactive interface of `widget-rs` outside of individual plugin business logic, built natively on the GPUI framework.

## Key Responsibilities

- **Dashboard (Main Window)**: Renders the central control center, including the left navigation sidebar, unified responsive layout system, and custom frameless titlebar.
- **Pages & Routing**: Implements primary view pages including the Home Dashboard, Widget Management catalog, and Global Application Settings.
- **Shared Component Library**: Provides reusable UI primitives such as buttons, toggles, cards, badges, and modals, ensuring strict visual consistency and refined micro-interactions.
- **Event Orchestration**: Integrates user interactions with `widget-app` for configuration persistence, window level toggles, and state transitions.

## UI Design Principles & Constraints

To maintain peak performance and aesthetic coherence:
- Views must strictly compose using the standard primitives provided in `layout::*` and `components::*`.
- In accordance with project design standards, user-facing UI elements must avoid raw emoji icons and instead utilize vector paths and stylized symbols.
- All pages delegate scrolling and container bounds to the unified Layout layer (`main_window.rs`).
