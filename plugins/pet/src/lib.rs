mod engine;
mod model;
mod view;

use gpui::*;
use widget_core::{AppConfig, Plugin};

use view::PetWidget;

pub struct PetPlugin;

impl Plugin for PetPlugin {
    fn id(&self) -> &'static str {
        "pet_plugin"
    }

    fn name(&self) -> &'static str {
        "桌面宠物"
    }

    fn description(&self) -> &'static str {
        "在桌面上养一只可爱的虚拟宠物，陪伴你每一天的工作时光。"
    }

    fn icon(&self) -> gpui_component::IconName {
        gpui_component::IconName::Star
    }

    fn version(&self) -> &'static str {
        "v0.1.0"
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let (x, y, w, h) = cx
            .try_global::<AppConfig>()
            .and_then(|cfg| cfg.plugins.get("pet_plugin").cloned())
            .map(|p| (p.x, p.y, p.width, p.height))
            .unwrap_or((100.0, 100.0, 256.0, 256.0));

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
            cx.new(|cx| PetWidget::new(window, cx))
        })
        .unwrap()
        .into()
    }
}

pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(PetPlugin)
}
