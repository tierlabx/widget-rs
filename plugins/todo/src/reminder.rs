use serde::{Deserialize, Serialize};

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
                // 如果 target_time_secs 是相对时长偏移（小于1年），显示为相对时长
                if *target_time_secs < 86400 * 365 {
                    let diff_mins = target_time_secs / 60;
                    if diff_mins < 60 {
                        format!("{} 分钟后", diff_mins.max(1))
                    } else {
                        let hours = diff_mins / 60;
                        let mins = diff_mins % 60;
                        format!("{}时{}分后", hours, mins)
                    }
                } else {
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

/// 预设提醒规则
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReminderPreset {
    pub id: String,
    pub label: String,
    pub rule: ReminderRule,
}

impl ReminderPreset {
    pub fn to_rule(&self) -> ReminderRule {
        match &self.rule {
            ReminderRule::Once { target_time_secs } => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if *target_time_secs < 86400 * 365 {
                    ReminderRule::Once {
                        target_time_secs: now + target_time_secs,
                    }
                } else {
                    ReminderRule::Once {
                        target_time_secs: *target_time_secs,
                    }
                }
            }
            other => other.clone(),
        }
    }
}

pub fn default_reminder_presets() -> Vec<ReminderPreset> {
    vec![
        ReminderPreset {
            id: "preset-30m".to_string(),
            label: "30分钟后".to_string(),
            rule: ReminderRule::Once {
                target_time_secs: 30 * 60,
            },
        },
        ReminderPreset {
            id: "preset-daily-18".to_string(),
            label: "每天 18:00".to_string(),
            rule: ReminderRule::Daily {
                minute_of_day: 18 * 60,
            },
        },
        ReminderPreset {
            id: "preset-weekly-fri".to_string(),
            label: "周五 17:00".to_string(),
            rule: ReminderRule::Weekly {
                weekday: 5,
                minute_of_day: 17 * 60,
            },
        },
        ReminderPreset {
            id: "preset-interval-30".to_string(),
            label: "每30分催办".to_string(),
            rule: ReminderRule::Interval { interval_mins: 30 },
        },
    ]
}
