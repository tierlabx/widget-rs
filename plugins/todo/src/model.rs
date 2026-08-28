use gpui::*;
use serde::{Deserialize, Serialize};
use widget_core::AppConfig;

/// 经典项目甘特色系定义
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
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

/// 智能提醒规则
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReminderRule {
    /// 单次定点提醒（目标 UNIX 秒级时间戳）
    Once { target_time_secs: u64 },
    /// 每天定时提醒（目标分钟：如 18:00 = 18*60 = 1080）
    Daily { minute_of_day: u32 },
    /// 每周定时提醒（weekday: 1=周一 .. 7=周日, minute_of_day）
    Weekly { weekday: u8, minute_of_day: u32 },
    /// 每月定时提醒（day_of_month: 1..31, minute_of_day）
    Monthly {
        day_of_month: u8,
        minute_of_day: u32,
    },
    /// 间隔循环催办（每隔 interval_mins 分钟提醒一次，直到任务标记完成）
    Interval { interval_mins: u32 },
}

impl ReminderRule {
    pub fn display_text(&self) -> String {
        match self {
            Self::Once { target_time_secs } => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if *target_time_secs <= now {
                    "已到期".to_string()
                } else {
                    let diff_mins = (*target_time_secs - now) / 60;
                    if diff_mins < 60 {
                        format!("{} 分钟后", diff_mins.max(1))
                    } else {
                        let hours = diff_mins / 60;
                        let mins = diff_mins % 60;
                        format!("{}时{}分后", hours, mins)
                    }
                }
            }
            Self::Daily { minute_of_day } => {
                format!("每天 {:02}:{:02}", minute_of_day / 60, minute_of_day % 60)
            }
            Self::Weekly {
                weekday,
                minute_of_day,
            } => {
                let w_str = match weekday {
                    1 => "周一",
                    2 => "周二",
                    3 => "周三",
                    4 => "周四",
                    5 => "周五",
                    6 => "周六",
                    _ => "周日",
                };
                format!(
                    "每{} {:02}:{:02}",
                    w_str,
                    minute_of_day / 60,
                    minute_of_day % 60
                )
            }
            Self::Monthly {
                day_of_month,
                minute_of_day,
            } => {
                format!(
                    "每月{}日 {:02}:{:02}",
                    day_of_month,
                    minute_of_day / 60,
                    minute_of_day % 60
                )
            }
            Self::Interval { interval_mins } => {
                format!("每 {} 分钟催办", interval_mins)
            }
        }
    }
}

/// 分类标签
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoTag {
    pub id: String,
    pub name: String,
    pub gantt_color: usize,
}

/// 单条待办任务
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub done: bool,
    #[serde(default)]
    pub tag_id: String,
    #[serde(default)]
    pub gantt_color: usize,
    #[serde(default)]
    pub reminder: Option<ReminderRule>,
    #[serde(default)]
    pub last_reminded_at: Option<u64>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// 待办数据总集（包含任务与分类标签）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoData {
    pub active_tag_id: String,
    pub tags: Vec<TodoTag>,
    pub items: Vec<TodoItem>,
}

impl Default for TodoData {
    fn default() -> Self {
        Self {
            active_tag_id: "all".to_string(),
            tags: vec![
                TodoTag {
                    id: "work".to_string(),
                    name: "工作".to_string(),
                    gantt_color: 0,
                },
                TodoTag {
                    id: "study".to_string(),
                    name: "学习".to_string(),
                    gantt_color: 1,
                },
                TodoTag {
                    id: "life".to_string(),
                    name: "生活".to_string(),
                    gantt_color: 2,
                },
                TodoTag {
                    id: "shopping".to_string(),
                    name: "购物".to_string(),
                    gantt_color: 3,
                },
            ],
            items: vec![
                TodoItem {
                    id: "1".to_string(),
                    text: "完成项目架构设计与评审".into(),
                    done: false,
                    tag_id: "work".into(),
                    gantt_color: 0,
                    reminder: Some(ReminderRule::Interval { interval_mins: 30 }),
                    last_reminded_at: None,
                    created_at: Some("今日 09:00".into()),
                },
                TodoItem {
                    id: "2".to_string(),
                    text: "每周五提交周报与总结".into(),
                    done: false,
                    tag_id: "work".into(),
                    gantt_color: 4,
                    reminder: Some(ReminderRule::Weekly {
                        weekday: 5,
                        minute_of_day: 17 * 60,
                    }),
                    last_reminded_at: None,
                    created_at: Some("今日 11:30".into()),
                },
                TodoItem {
                    id: "3".to_string(),
                    text: "阅读 Rust 高级异步编程章节".into(),
                    done: true,
                    tag_id: "study".into(),
                    gantt_color: 1,
                    reminder: None,
                    last_reminded_at: None,
                    created_at: Some("今日 14:00".into()),
                },
            ],
        }
    }
}

pub struct TodoModel;

impl TodoModel {
    pub fn load(cx: &mut App) -> TodoData {
        if let Some(cfg) = cx.try_global::<AppConfig>() {
            // 尝试读取新版 TodoData
            if let Some(data) = cfg.get_plugin_data::<TodoData>("todo_widget") {
                return data;
            }
            // 尝试迁移旧版 Vec<TodoItem>
            if let Some(old_items) = cfg.get_plugin_data::<Vec<TodoItem>>("todo_widget") {
                let mut data = TodoData::default();
                data.items = old_items;
                return data;
            }
        }
        TodoData::default()
    }

    pub fn save(data: &TodoData, cx: &mut App) {
        cx.update_global::<AppConfig, _>(|cfg, _| {
            cfg.set_plugin_data("todo_widget", data);
        });
        widget_core::save_config_now(cx);
    }
}
