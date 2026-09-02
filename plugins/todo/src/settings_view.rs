use gpui::*;
use widget_core::render_settings_shell;

use crate::model::{ReminderRule, TodoData, TodoModel};
use crate::preset_editor::{render_add_card, render_preset_list};

/// 待办事项插件独立设置视图
pub struct TodoSettingsView {
    pub data: TodoData,
    pub new_preset_type: usize, // 0: 相对时间(分钟后), 1: 每日定时, 2: 间隔催办
    pub new_relative_mins: u32,
    pub new_daily_hour: u32,
    pub new_daily_min: u32,
    pub new_interval_mins: u32,
}

impl TodoSettingsView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = TodoModel::load(cx);
        Self {
            data,
            new_preset_type: 0,
            new_relative_mins: 15,
            new_daily_hour: 9,
            new_daily_min: 0,
            new_interval_mins: 25,
        }
    }

    pub fn save(&mut self, cx: &mut Context<Self>) {
        TodoModel::save(&self.data, cx);
        cx.set_global(crate::TodoDataReloadTrigger(true));
        cx.notify();
        cx.refresh_windows();
    }

    pub fn add_current_preset(&mut self, cx: &mut Context<Self>) {
        let (label, rule) = match self.new_preset_type {
            0 => {
                let mins = self.new_relative_mins.max(1);
                (
                    format!("{mins}分钟后"),
                    ReminderRule::Once {
                        target_time_secs: mins as u64 * 60,
                    },
                )
            }
            1 => {
                let h = self.new_daily_hour % 24;
                let m = self.new_daily_min % 60;
                (
                    format!("每天 {:02}:{:02}", h, m),
                    ReminderRule::Daily {
                        minute_of_day: h * 60 + m,
                    },
                )
            }
            _ => {
                let mins = self.new_interval_mins.max(5);
                (
                    format!("每{mins}分催办"),
                    ReminderRule::Interval {
                        interval_mins: mins,
                    },
                )
            }
        };

        self.data.add_preset(label, rule);
        self.save(cx);
    }
}

impl Render for TodoSettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = div()
            .flex()
            .flex_col()
            .p(px(16.0))
            .gap(px(14.0))
            .child(render_preset_list(self, cx))
            .child(render_add_card(self, cx))
            .child(
                div()
                    .pt(px(10.0))
                    .border_t_1()
                    .border_color(rgb(0x21262d))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .id("save-btn")
                            .w_full()
                            .py(px(9.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(7.0))
                            .bg(rgb(0x00d992))
                            .hover(|s| s.bg(rgba(0x00d992ccu32)))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, win, cx| {
                                this.save(cx);
                                win.remove_window();
                                cx.defer(|_| {
                                    widget_core::trim_process_memory();
                                });
                            }))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x050507))
                                    .child("保存并关闭"),
                            ),
                    )
                    .child(
                        div()
                            .id("test-notification-btn")
                            .w_full()
                            .py(px(8.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .gap(px(6.0))
                            .rounded(px(7.0))
                            .border_1()
                            .border_color(rgba(0x38bdf860u32))
                            .bg(rgba(0x38bdf80du32))
                            .hover(|s| s.bg(rgba(0x38bdf820u32)))
                            .cursor_pointer()
                            .on_click(cx.listener(|_, _, _, _| {
                                crate::notification::send_todo_notification(
                                    "待办事项提醒测试",
                                    "这是一条测试提醒通知，系统通知通道工作正常！",
                                );
                            }))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x38bdf8))
                                    .child("发送测试通知"),
                            ),
                    ),
            );

        render_settings_shell("待办事项 - 设置", content)
    }
}
