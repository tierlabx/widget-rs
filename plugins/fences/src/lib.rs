mod dialog;
pub mod icon_extractor;
mod item_card;
mod model;
mod view;

use gpui::*;
use widget_core::Plugin;

use view::FencesWidget;

pub struct FencesWidgetPlugin;

impl Plugin for FencesWidgetPlugin {
    fn id(&self) -> &'static str {
        "fences_widget"
    }

    fn name(&self) -> &'static str {
        "桌面收纳"
    }

    fn description(&self) -> &'static str {
        "半透明磨砂桌面格子，分类收纳文件、文件夹与应用程序。"
    }

    fn icon(&self) -> gpui_component::IconName {
        gpui_component::IconName::LayoutDashboard
    }

    fn estimated_memory(&self) -> usize {
        2 * 1024 * 1024
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = widget_core::default_widget_window_options(
            cx,
            "fences_widget",
            (850.0, 50.0, 360.0, 480.0),
        );

        cx.open_window(options, |window, cx| {
            let content = cx.new(|cx| FencesWidget::new(window, cx));
            let widget_window = cx.new(|_cx| widget_core::WidgetWindow::new(content));
            cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
        })
        .unwrap()
        .into()
    }
}

/// 标准插件入口函数，供 widget-cli 和主程序注入使用
pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(FencesWidgetPlugin)
}
