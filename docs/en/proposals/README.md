# Plugin Proposals & Incubation System

<p align="center">
  <strong>English</strong> | <a href="../../proposals/README.md">简体中文</a>
</p>

This directory contains functional design specifications, architectural research, and implementation roadmaps for new `widget-rs` plugins and widgets, serving as the central design hub from concept to production.

---

## 1. Proposal Lifecycle

Each new widget or refactoring proposal progresses through the following stages:

```
[ 1. Concept & Draft ] 
       ↓ 
[ 2. RFC & Architecture Review ] 
       ↓ 
[ 3. Core Prototype ] 
       ↓ 
[ 4. Plugin Implementation (in plugins/) ] 
       ↓ 
[ 5. Stable Release ]
```

- **Draft**: Defines motivation, user pain points, interaction models, and planned capabilities.
- **Review / RFC**: Details technical architecture, rendering pipeline, data flows, state machines, and `widget-core` integration contracts.
- **Prototype**: Validates critical feasibility hurdles (e.g. Win32 hooks, GPUI custom paint paths, or shaders) in isolated modules.
- **Implementation**: Formally scaffolds a crate in `plugins/` via `widget-cli` and integrates into the app.
- **Release**: Published in general releases.

---

## 2. Directory Structure Conventions

Each new widget proposal creates an isolated subdirectory under `docs/proposals/<plugin-slug>/` (and mirrored in `docs/en/proposals/<plugin-slug>/`), registered in the index table below.

```text
docs/en/proposals/<plugin-slug>/
├── README.md                          # Overview, positioning, and milestones
├── architecture-and-rendering.md      # UI & GPUI rendering pipeline / animation design
├── smtc-and-lyrics-engine.md          # Core business engine & protocol integration (if applicable)
└── window-and-integration.md          # Window hosting, persistence, and spec compliance
```

---

## 3. Strict Design Redlines

All proposals must satisfy these non-negotiable architectural constraints:

1. **Desktop Persistence (`Win+D` Resident)**:
   - Must mount to system `Progman` on creation so widgets remain on the desktop when the user invokes `Win+D`.
2. **Standard Container Wrapping**:
   - Must be wrapped inside `widget_core::WidgetWindow<T>`. Never manually implement drag bars or edit borders.
3. **Dedicated Settings Standard**:
   - Settings modals must use `widget_core::render_settings_shell`, `settings_card`, and `settings_section_header`.
4. **File Length & Performance**:
   - No single `.rs` file may exceed **400 lines**.
   - Must sustain 60Hz/120Hz V-Sync with base memory usage kept within 50MB.
5. **UI Emojis Prohibited**:
   - User-facing widget UIs must not use raw emoji icons; use vector paths or geometric symbols.

---

## 4. Active Proposals

| Proposal ID | Name | Directory Link | Current Phase | Key Highlights |
| :--- | :--- | :--- | :--- | :--- |
| **PROP-001** | **Immersive Desktop Folia Lyrics** | [folia-lyrics](./folia-lyrics/README.md) | **Review / RFC** | Typography Music Video visuals, SMTC media tracking, per-word karaoke pop, dynamic glow stage, desktop resident |
