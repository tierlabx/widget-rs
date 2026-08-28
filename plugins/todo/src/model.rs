use gpui::*;
use serde::{Deserialize, Serialize};
use widget_core::AppConfig;

/// 经典项目甘特色系定义
#[derive(Clone, Copy, Debug)]
pub struct GanttColor {
    pub name: &'static str,
    pub hex: u32,
    pub bg_alpha_hex: u32,
}

pub const GANTT_COLORS: &[GanttColor] = &[
    GanttColor {
        name: "标准蓝",
        hex: 0x38bdf8,
        bg_alpha_hex: 0x38bdf825,
    }, // 天空蓝
    GanttColor {
        name: "进行绿",
        hex: 0x34d399,
        bg_alpha_hex: 0x34d39925,
    }, // 翡翠绿
    GanttColor {
        name: "核心紫",
        hex: 0xa78bfa,
        bg_alpha_hex: 0xa78bfa25,
    }, // 薰衣紫
    GanttColor {
        name: "关注橙",
        hex: 0xfb923c,
        bg_alpha_hex: 0xfb923c25,
    }, // 活力橙
    GanttColor {
        name: "紧急红",
        hex: 0xf87171,
        bg_alpha_hex: 0xf8717125,
    }, // 珊瑚红
    GanttColor {
        name: "规划金",
        hex: 0xfacc15,
        bg_alpha_hex: 0xfacc1525,
    }, // 日光金
];

/// 单条待办任务
#[derive(Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub text: String,
    pub done: bool,
    #[serde(default)]
    pub gantt_color: usize,
    #[serde(default)]
    pub created_at: Option<String>,
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
                        gantt_color: 0,
                        created_at: Some("今天 09:00".into()),
                    },
                    TodoItem {
                        text: "编写文档".into(),
                        done: true,
                        gantt_color: 1,
                        created_at: Some("今天 11:30".into()),
                    },
                    TodoItem {
                        text: "代码审查与发布".into(),
                        done: false,
                        gantt_color: 4,
                        created_at: Some("今天 14:00".into()),
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
