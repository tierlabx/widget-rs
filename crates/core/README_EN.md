# Widget RS Core Crate

<p align="center">
  <strong>English</strong> | <a href="README.md">简体中文</a>
</p>

`widget-core` is the foundational common abstraction layer of `widget-rs`, containing core data structures, shared traits, and standard container wrappers.

## Key Responsibilities

- **Shared Data Structures**: Application-level global configuration (`AppConfig`), plugin metadata, and entity abstractions.
- **Plugin Interface Definitions**: Core contracts for widget plugin development, including `Plugin` and `WidgetContent` traits.
- **Unified Container Wrapper** (`WidgetWindow<T>`): Framework-level widget container that encapsulates edit-mode detection, drag handles, border highlights, and native window style management.
- **UI Primitives & Synchronization**: Provides atomic coordination flags (such as `NATIVE_EDIT_MODE`) ensuring thread-safe state synchronization across `app`, `ui`, and various plugin crates.
- **Cross-Module Decoupling**: Acts as a bridge between the application runtime and feature/UI layers.

## Design Principles

This module strictly maintains minimal external dependencies. It avoids direct operating-system platform calls or monolithic rendering libraries, ensuring any widget or extension module can build upon `core` with maximum portability and testability.
