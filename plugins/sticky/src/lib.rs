mod model;
#[cfg(test)]
mod model_tests;
mod view;

use gpui::*;
use widget_core::Plugin;

use view::StickyWidget;

pub struct StickyWidgetPlugin;

impl Plugin for StickyWidgetPlugin {
    fn id(&self) -> &'static str {
        "sticky_widget"
    }

    fn name(&self) -> &'static str {
        "极客便签"
    }

    fn description(&self) -> &'static str {
        "将随手记下的灵感、备忘录以极客形式贴在桌面。"
    }

    fn icon(&self) -> gpui_component::IconName {
        gpui_component::IconName::File
    }

    fn estimated_memory(&self) -> usize {
        // 便签预估内存：基础 2MB + 图片缓存预估
        2 * 1024 * 1024
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = widget_core::default_widget_window_options(
            cx,
            "sticky_widget",
            (1250.0, 50.0, 320.0, 360.0),
        );

        cx.open_window(options, |window, cx| {
            let content = cx.new(|cx| StickyWidget::new(window, cx));
            let widget_window = cx.new(|_cx| widget_core::WidgetWindow::new(content));
            cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
        })
        .unwrap()
        .into()
    }
}

/// 标准插件入口函数，供 widget-cli 和主程序注入使用
pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(StickyWidgetPlugin)
}
