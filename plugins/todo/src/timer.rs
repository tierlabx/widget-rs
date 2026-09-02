use gpui::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::model::{ReminderRule, TodoModel};
use crate::view::TodoWidget;

pub fn get_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn get_simple_time_str() -> String {
    let secs = get_now_secs();
    let local_secs = (secs + 28800) % 86400; // UTC+8
    let hours = local_secs / 3600;
    let mins = (local_secs % 3600) / 60;
    format!("{:02}:{:02}", hours, mins)
}

/// 启动待办事项智能定时提醒检查后台任务
pub fn spawn_todo_timer(
    this_weak: WeakEntity<TodoWidget>,
    cx: &mut Context<TodoWidget>,
) -> gpui::Task<()> {
    let app_cx: &mut App = cx;
    app_cx.spawn(async move |async_cx| loop {
        async_cx
            .background_executor()
            .timer(Duration::from_secs(2))
            .await;

        let res = async_cx.update(|cx| {
            let _ = this_weak.update(cx, |this, cx| {
                // 检测设置面板是否修改并保存了预设/标签数据
                let should_reload = cx
                    .try_global::<crate::TodoDataReloadTrigger>()
                    .map(|t| t.0)
                    .unwrap_or(false);
                if should_reload {
                    cx.set_global(crate::TodoDataReloadTrigger(false));
                    let latest_data = TodoModel::load(cx);
                    this.data_mut().reminder_presets = latest_data.reminder_presets;
                    this.data_mut().tags = latest_data.tags;
                    cx.notify();
                }

                let now = get_now_secs();
                let local_secs = (now + 28800) % 86400;
                let curr_minute_of_day = (local_secs / 60) as u32;

                let tags = this.data().tags.clone();
                let mut needs_save = false;
                for item in &mut this.data_mut().items {
                    if item.done {
                        continue;
                    }
                    if let Some(rule) = &item.reminder {
                        let should_remind = match rule {
                            ReminderRule::Once { target_time_secs } => {
                                now >= *target_time_secs && item.last_reminded_at.is_none()
                            }
                            ReminderRule::Daily { minute_of_day } => {
                                curr_minute_of_day == *minute_of_day
                                    && item
                                        .last_reminded_at
                                        .map(|t| now.saturating_sub(t) > 60)
                                        .unwrap_or(true)
                            }
                            ReminderRule::Weekly { minute_of_day, .. } => {
                                curr_minute_of_day == *minute_of_day
                                    && item
                                        .last_reminded_at
                                        .map(|t| now.saturating_sub(t) > 60)
                                        .unwrap_or(true)
                            }
                            ReminderRule::Monthly { minute_of_day, .. } => {
                                curr_minute_of_day == *minute_of_day
                                    && item
                                        .last_reminded_at
                                        .map(|t| now.saturating_sub(t) > 60)
                                        .unwrap_or(true)
                            }
                            ReminderRule::Interval { interval_mins } => item
                                .last_reminded_at
                                .map(|t| now.saturating_sub(t) >= (*interval_mins as u64 * 60))
                                .unwrap_or(true),
                        };

                        if should_remind {
                            item.last_reminded_at = Some(now);
                            needs_save = true;

                            let tag_name = tags
                                .iter()
                                .find(|t| t.id == item.tag_id)
                                .map(|t| t.name.as_str());

                            let title = if let Some(tag) = tag_name {
                                format!("待办事项提醒 · {tag}")
                            } else {
                                "待办事项提醒".to_string()
                            };
                            let body = item.text.clone();
                            crate::notification::send_todo_notification(&title, &body);
                        }
                    }
                }
                if needs_save {
                    TodoModel::save(this.data(), cx);
                    cx.notify();
                }
            });
        });

        if res.is_err() {
            break;
        }
    })
}
