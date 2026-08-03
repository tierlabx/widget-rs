use gpui::*;
use widget_core::AppConfig;

pub struct StickyModel;

impl StickyModel {
    pub fn load(cx: &mut App) -> String {
        cx.try_global::<AppConfig>()
            .and_then(|c| c.get_plugin_data::<String>("sticky_widget"))
            .unwrap_or_default()
    }

    pub fn save(text: &str, cx: &mut App) {
        cx.update_global::<AppConfig, _>(|config, _| {
            config.set_plugin_data("sticky_widget", &text);
        });
        widget_core::save_config_now(cx);
    }
}
