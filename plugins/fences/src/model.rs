use gpui::*;
use serde::{Deserialize, Serialize};
use widget_core::AppConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FenceItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FenceCategory {
    pub name: String,
    pub items: Vec<FenceItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FencesData {
    pub active_category: usize,
    pub categories: Vec<FenceCategory>,
}

impl Default for FencesData {
    fn default() -> Self {
        Self {
            active_category: 0,
            categories: vec![FenceCategory {
                name: "常用".to_string(),
                items: vec![],
            }],
        }
    }
}

pub struct FencesModel;

impl FencesModel {
    pub fn load(cx: &mut App) -> FencesData {
        cx.try_global::<AppConfig>()
            .and_then(|cfg| cfg.get_plugin_data::<FencesData>("fences_widget"))
            .unwrap_or_default()
    }

    pub fn save(data: &FencesData, cx: &mut App) {
        cx.update_global::<AppConfig, _>(|cfg, _| {
            cfg.set_plugin_data("fences_widget", data);
        });
        widget_core::save_config_now(cx);
    }
}
