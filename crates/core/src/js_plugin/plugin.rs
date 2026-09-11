use gpui::*;
use std::path::Path;

use crate::js_plugin::manifest::JsWidgetManifest;
use crate::js_plugin::view::JsWidgetContent;
use crate::{
    default_widget_window_options, default_widget_window_options_blurred, Plugin, WidgetWindow,
};

/// 动态 JavaScript 小部件插件实例
pub struct JsPlugin {
    manifest: JsWidgetManifest,
}

impl JsPlugin {
    /// 从插件目录加载实例化
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let manifest = JsWidgetManifest::load_from_dir(dir)?;
        Ok(Self { manifest })
    }

    /// 从已解析的清单构建
    pub fn new(manifest: JsWidgetManifest) -> Self {
        Self { manifest }
    }

    pub fn manifest(&self) -> &JsWidgetManifest {
        &self.manifest
    }
}

impl Plugin for JsPlugin {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn description(&self) -> &str {
        &self.manifest.description
    }

    fn icon(&self) -> gpui_component::IconName {
        self.manifest.parse_icon()
    }

    fn version(&self) -> &str {
        &self.manifest.version
    }

    fn author(&self) -> &str {
        &self.manifest.author
    }

    fn estimated_memory(&self) -> usize {
        1024 * 1024 * 3 // 预估约 3MB
    }

    fn is_external(&self) -> bool {
        true
    }

    fn has_settings(&self) -> bool {
        self.manifest.has_settings
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let manifest = self.manifest.clone();
        let plugin_id = manifest.id.clone();
        let default_size = (200.0, 200.0, manifest.window.width, manifest.window.height);

        let window_options = if manifest.window.blurred {
            default_widget_window_options_blurred(cx, &plugin_id, default_size)
        } else {
            default_widget_window_options(cx, &plugin_id, default_size)
        };

        cx.open_window(window_options, move |_window, cx| {
            let content = cx.new(|cx| JsWidgetContent::new(manifest, cx));
            cx.new(|_| WidgetWindow::new(content))
        })
        .expect("创建 JS 小组件窗口失败")
        .into()
    }
}
