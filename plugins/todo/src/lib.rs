mod model;
mod view;

use gpui::*;
use widget_core::Plugin;

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
        let (x, y, w, h) =
            widget_core::resolve_plugin_bounds(cx, "todo_widget", (1250.0, 450.0, 360.0, 460.0));

        println!("[TodoPlugin] 初始位置: ({}, {}) {}x{}", x, y, w, h);

        let options = WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::PopUp,
            is_resizable: false,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::new(px(x), px(y)),
                size(px(w), px(h)),
            ))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| TodoWidget::new(window, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        })
        .unwrap()
        .into()
    }
}

/// 标准插件入口函数，供 widget-cli 和主程序注入使用
pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(TodoWidgetPlugin)
}
