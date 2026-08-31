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

impl TodoData {
    /// 新增分类标签，返回新标签 ID
    pub fn add_tag(&mut self, name: String, gantt_color: usize) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let new_id = format!("tag-{}", now);
        self.tags.push(TodoTag {
            id: new_id.clone(),
            name,
            gantt_color,
        });
        new_id
    }

    /// 更新分类标签名称与颜色
    pub fn update_tag(&mut self, tag_id: &str, name: String, gantt_color: usize) -> bool {
        if let Some(tag) = self.tags.iter_mut().find(|t| t.id == tag_id) {
            tag.name = name;
            tag.gantt_color = gantt_color;
            true
        } else {
            false
        }
    }

    /// 安全删除分类标签，并将该标签下的任务迁移到首个可用标签
    pub fn delete_tag_and_migrate(&mut self, tag_id: &str) -> bool {
        if self.tags.len() <= 1 {
            return false;
        }

        if let Some(pos) = self.tags.iter().position(|t| t.id == tag_id) {
            self.tags.remove(pos);

            let fallback_tag_id = self
                .tags
                .first()
                .map(|t| t.id.clone())
                .unwrap_or_else(|| "work".to_string());

            // 迁移该分类下的条目
            for item in &mut self.items {
                if item.tag_id == tag_id {
                    item.tag_id = fallback_tag_id.clone();
                }
            }

            // 若当前处于被删除分类的视图下，重置为全部
            if self.active_tag_id == tag_id {
                self.active_tag_id = "all".to_string();
            }

            true
        } else {
            false
        }
    }
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
            items: vec![],
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
                return TodoData {
                    items: old_items,
                    ..Default::default()
                };
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
