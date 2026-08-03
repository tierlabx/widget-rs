use gpui::*;
use serde::{Deserialize, Serialize};
use widget_core::AppConfig;

/// 单条待办任务
#[derive(Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub text: String,
    pub done: bool,
}

pub struct TodoModel;

impl TodoModel {
    pub fn load(cx: &mut App) -> Vec<TodoItem> {
        cx.try_global::<AppConfig>()
            .and_then(|cfg| cfg.get_plugin_data::<Vec<TodoItem>>("todo_widget"))
            .unwrap_or_else(|| {
                // 首次启动使用示例数据
                vec![
                    TodoItem {
                        text: "完成项目设计".into(),
                        done: false,
                    },
                    TodoItem {
                        text: "编写文档".into(),
                        done: true,
                    },
                    TodoItem {
                        text: "代码审查".into(),
                        done: false,
                    },
                ]
            })
    }

    pub fn save(items: &[TodoItem], cx: &mut App) {
        cx.update_global::<AppConfig, _>(|cfg, _| {
            cfg.set_plugin_data("todo_widget", &items);
        });
        widget_core::save_config_now(cx);
    }
}
