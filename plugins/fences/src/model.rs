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
    #[serde(default)]
    pub collapsed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FencesData {
    pub categories: Vec<FenceCategory>,
}

impl Default for FencesData {
    fn default() -> Self {
        Self {
            categories: vec![
                FenceCategory {
                    name: "程序".to_string(),
                    items: vec![],
                    collapsed: false,
                },
                FenceCategory {
                    name: "文件夹".to_string(),
                    items: vec![],
                    collapsed: false,
                },
                FenceCategory {
                    name: "文件".to_string(),
                    items: vec![],
                    collapsed: false,
                },
            ],
        }
    }
}

pub struct FencesModel;

impl FencesModel {
    pub fn load(cx: &mut App) -> FencesData {
        let mut data = cx
            .try_global::<AppConfig>()
            .and_then(|cfg| cfg.get_plugin_data::<FencesData>("fences_widget"))
            .unwrap_or_default();

        // 确保必须包含三栏：程序、文件夹、文件
        if data.categories.len() < 3 {
            data = FencesData::default();
        }
        data
    }

    pub fn save(data: &FencesData, cx: &mut App) {
        cx.update_global::<AppConfig, _>(|cfg, _| {
            cfg.set_plugin_data("fences_widget", data);
        });
        widget_core::save_config_now(cx);
    }
}
