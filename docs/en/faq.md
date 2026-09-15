# Frequently Asked Questions (FAQ)

<p align="center">
  <strong>English</strong> | <a href="../常见问题_FAQ.md">简体中文</a>
</p>

This page collects answers to common questions asked by Widget-RS users and developers.

---

## General Usage Questions

### Q: A widget is blocking other windows. How do I disable Always on Top?
**A**: Open the Dashboard, navigate to the widget card or "Global Settings" in the sidebar, locate the "Always on Top" toggle, and turn it off.

### Q: I cannot click desktop icons or text underneath a widget?
**A**: Check whether "Mouse Passthrough" (click-through) is enabled. When enabled, all mouse clicks pass directly to the underlying window. Right-click the system tray icon and uncheck "Enable Mouse Passthrough" to restore normal interaction.

### Q: How do I move frameless sticky notes or todo widgets?
**A**: Right-click the system tray icon and select "Edit Layout" (or toggle Edit Mode in the Dashboard). A green drag handle (`#00d992`) will appear above each widget. Click and drag this handle to reposition widgets.

---

## Developer & Compilation Questions

### Q: Windows build fails with `link.exe` not found?
**A**: Ensure you have installed Visual Studio Build Tools with the "Desktop development with C++" workload checked.

### Q: Why isn't there any window dragging or border code in a plugin's `view.rs`?
**A**: Because `widget-rs`'s underlying `WidgetWindow` container centrally handles window chrome, resizing borders, and edit mode transitions. Plugins only implement the `WidgetContent` trait to render their core UI. See [Widget Specification](widget-spec.md) and [Plugin Development Guide](plugin-development-guide.md).

### Q: Can I add a permanent custom titlebar to my widget?
**A**: **Not recommended**. The project adheres to an immersive minimalist aesthetic (VoltAgent style). Permanent window titlebars are discouraged in favor of frameless desktop widgets.

### Q: How do I contribute code?
**A**: Please review [CONTRIBUTING.md](../../CONTRIBUTING_EN.md). When submitting a pull request, adhere to Conventional Commits (e.g., `feat(sticky): add color palette`).
