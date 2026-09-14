use gpui::*;
use gpui_shell::ShellRoot;

use crate::js_plugin::manifest::JsWidgetManifest;
use crate::WidgetContent;

/// 托管并渲染 gpui-shell 根视图的通用小组件内容容器
pub struct JsWidgetContent {
    manifest: JsWidgetManifest,
    root: Entity<ShellRoot>,
}

impl JsWidgetContent {
    pub fn new(manifest: JsWidgetManifest, root: Entity<ShellRoot>) -> Self {
        Self { manifest, root }
    }

    pub fn manifest(&self) -> &JsWidgetManifest {
        &self.manifest
    }

    pub fn root(&self) -> &Entity<ShellRoot> {
        &self.root
    }
}

impl WidgetContent for JsWidgetContent {
    fn plugin_id(&self) -> &str {
        &self.manifest.id
    }

    fn drag_label(&self) -> &str {
        &self.manifest.name
    }

    fn show_drag_handle(&self) -> bool {
        true
    }
}

impl Render for JsWidgetContent {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(self.root.clone())
    }
}
