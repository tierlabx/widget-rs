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
            categories: vec![
                FenceCategory {
                    name: "程序".to_string(),
                    items: vec![
                        FenceItem {
                            name: "资源管理器".to_string(),
                            path: "explorer.exe".to_string(),
                            is_dir: false,
                        },
                        FenceItem {
                            name: "记事本".to_string(),
                            path: "notepad.exe".to_string(),
                            is_dir: false,
                        },
                        FenceItem {
                            name: "计算器".to_string(),
                            path: "calc.exe".to_string(),
                            is_dir: false,
                        },
                        FenceItem {
                            name: "任务管理器".to_string(),
                            path: "taskmgr.exe".to_string(),
                            is_dir: false,
                        },
                    ],
                },
                FenceCategory {
                    name: "文件夹".to_string(),
                    items: vec![
                        FenceItem {
                            name: "桌面".to_string(),
                            path: get_user_folder("Desktop"),
                            is_dir: true,
                        },
                        FenceItem {
                            name: "下载".to_string(),
                            path: get_user_folder("Downloads"),
                            is_dir: true,
                        },
                        FenceItem {
                            name: "文档".to_string(),
                            path: get_user_folder("Documents"),
                            is_dir: true,
                        },
                    ],
                },
                FenceCategory {
                    name: "文件和文档".to_string(),
                    items: vec![
                        FenceItem {
                            name: "项目设计".to_string(),
                            path: "docs/详细设计.md".to_string(),
                            is_dir: false,
                        },
                        FenceItem {
                            name: "小组件规范".to_string(),
                            path: "docs/小组件规范.md".to_string(),
                            is_dir: false,
                        },
                    ],
                },
            ],
        }
    }
}

fn get_user_folder(sub: &str) -> String {
    std::env::var("USERPROFILE")
        .map(|p| format!("{}\\{}", p, sub))
        .unwrap_or_else(|_| "C:\\".to_string())
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
