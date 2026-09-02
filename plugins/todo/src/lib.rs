mod content_panel;
mod item_card;
mod item_detail;
mod model;
#[cfg(test)]
mod model_tests;
mod notification;
mod preset_editor;
mod settings_view;
mod sidebar;
mod tag_modal;
mod timer;
mod view;

use gpui::*;
use widget_core::Plugin;

use settings_view::TodoSettingsView;
use view::TodoWidget;

pub struct TodoWidgetPlugin;

impl Plugin for TodoWidgetPlugin {
    fn id(&self) -> &'static str {
        "todo_widget"
    }

    fn name(&self) -> &'static str {
        "待办事项"
    }

    fn description(&self) -> &'static str {
        "极简高效的任务清单，助你轻松掌控今日核心目标。"
    }

    fn icon(&self) -> gpui_component::IconName {
        gpui_component::IconName::CircleCheck
    }

    fn estimated_memory(&self) -> usize {
        // 待办事项预估内存：基础 1.5MB
        1536 * 1024
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = widget_core::default_widget_window_options(
            cx,
            "todo_widget",
            (1250.0, 450.0, 360.0, 460.0),
        );

        cx.open_window(options, |window, cx| {
            let content = cx.new(|cx| TodoWidget::new(window, cx));
            let widget_window = cx.new(|_cx| widget_core::WidgetWindow::new(content));
            cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
        })
        .unwrap()
        .into()
    }

    fn build_settings_window(&self, cx: &mut App) {
        let options = widget_core::default_settings_window_options(cx, (420.0, 560.0));

        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| TodoSettingsView::new(window, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        })
        .unwrap();
    }

    fn has_settings(&self) -> bool {
        true
    }
}

/// 标准插件入口函数，供 widget-cli 和主程序注入使用
pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(TodoWidgetPlugin)
}
