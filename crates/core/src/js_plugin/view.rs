use gpui::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::js_plugin::manifest::JsWidgetManifest;
use crate::js_plugin::runtime::JsEngine;
use crate::WidgetContent;

/// 动态 JavaScript 小部件视图内容组件
pub struct JsWidgetContent {
    manifest: JsWidgetManifest,
    engine: Arc<Mutex<JsEngine>>,
    _timer: Option<Task<()>>,
}

impl JsWidgetContent {
    pub fn new(manifest: JsWidgetManifest, cx: &mut Context<Self>) -> Self {
        let engine = Arc::new(Mutex::new(JsEngine::new(manifest.clone())));

        // 若设置了定时刷新间隔，启动后台定时器触发 cx.notify()
        let interval = manifest.refresh_interval_ms;
        let timer = if interval > 0 {
            let entity_weak = cx.entity().downgrade();
            let app_cx: &mut App = cx;
            Some(app_cx.spawn(async move |async_cx| loop {
                async_cx
                    .background_executor()
                    .timer(Duration::from_millis(interval))
                    .await;
                let active = async_cx.update(|cx| {
                    if let Some(entity) = entity_weak.upgrade() {
                        entity.update(cx, |_, cx| {
                            cx.notify();
                        });
                        true
                    } else {
                        false
                    }
                });
                if !active {
                    break;
                }
            }))
        } else {
            None
        };

        Self {
            manifest,
            engine,
            _timer: timer,
        }
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
        let node = if let Ok(engine) = self.engine.lock() {
            engine.render_frame()
        } else {
            crate::js_plugin::model::JsRenderNode::new_text("扩展加载失败", 14.0, "#ef4444")
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(node.into_element())
    }
}
