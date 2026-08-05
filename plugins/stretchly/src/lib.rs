mod model;
mod tips;
mod view;

use gpui::*;
use widget_core::{AppConfig, Plugin};

use view::StretchlyWidget;

mod settings_view;
pub use settings_view::StretchlyLiveStats;
use settings_view::StretchlySettingsView;

/// 设置页通知组件「立即应用配置并重置计时器」的全局信号
pub struct StretchlyApplyNow(pub bool);
impl Global for StretchlyApplyNow {}

/// 全局信号：休息是否正在进行（供副屏遮罩窗口监听自毁）
pub struct StretchlyBreakActive(pub bool);
impl Global for StretchlyBreakActive {}

pub struct StretchlyWidgetPlugin;

impl Plugin for StretchlyWidgetPlugin {
    fn id(&self) -> &'static str {
        "stretchly_widget"
    }

    fn name(&self) -> &'static str {
        "休息提醒"
    }

    fn description(&self) -> &'static str {
        "定期提醒你休息，保护视力和身体健康。"
    }

    fn icon(&self) -> gpui_component::IconName {
        gpui_component::IconName::CircleCheck
    }

    fn estimated_memory(&self) -> usize {
        1024 * 1024 // 1MB
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let (x, y, w, h) = cx
            .try_global::<AppConfig>()
            .and_then(|cfg| cfg.plugins.get("stretchly_widget").cloned())
            .map(|p| (p.x, p.y, p.width, p.height))
            // 默认：右上角紧凑小组件，280×100
            .unwrap_or((1250.0, 100.0, 280.0, 100.0));

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
            let view = cx.new(|cx| StretchlyWidget::new(window, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        })
        .unwrap()
        .into()
    }

    fn build_settings_window(&self, cx: &mut App) {
        let options = WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Opaque,
            kind: WindowKind::PopUp,
            is_resizable: true,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::new(px(200.0), px(200.0)),
                size(px(420.0), px(740.0)),
            ))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| StretchlySettingsView::new(window, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        })
        .unwrap();
    }
}

pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(StretchlyWidgetPlugin)
}
