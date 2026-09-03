use gpui::*;
use serde::{Deserialize, Serialize};
use widget_core::AppConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FenceItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

impl FenceItem {
    #[allow(dead_code)]
    pub fn is_web_url(&self) -> bool {
        self.path.starts_with("http://") || self.path.starts_with("https://")
    }
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

        if data.categories.is_empty() {
            data = FencesData::default();
        } else if data.categories.len() > 3 {
            // 平滑兼容迁移：若原配置包含多余栏目（如网页书签栏），合并其所有项至第0栏“程序”
            let mut extra_items = Vec::new();
            for extra_cat in data.categories.drain(3..) {
                extra_items.extend(extra_cat.items);
            }
            if let Some(first_cat) = data.categories.get_mut(0) {
                first_cat.items.extend(extra_items);
            }
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

#[cfg(test)]
mod tests;
