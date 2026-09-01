use gpui::*;
use gpui_component::{Icon, IconName};
use widget_core::{render_settings_shell, settings_card, settings_section_header};

use crate::model::{ReminderRule, TodoData, TodoModel};

/// 待办事项插件独立设置视图
pub struct TodoSettingsView {
    pub data: TodoData,
    new_preset_type: usize, // 0: 相对时间(分钟后), 1: 每日定时, 2: 间隔催办
    new_relative_mins: u32,
    new_daily_hour: u32,
    new_daily_min: u32,
    new_interval_mins: u32,
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

    fn save(&mut self, cx: &mut Context<Self>) {
        TodoModel::save(&self.data, cx);
        cx.notify();
    }

    fn add_current_preset(&mut self, cx: &mut Context<Self>) {
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

    fn render_preset_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let presets = self.data.reminder_presets.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(settings_section_header("提醒预设管理"))
            .child(if presets.is_empty() {
                settings_card()
                    .p(px(14.0))
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x8b949e))
                            .child("暂无预设，请在下方添加"),
                    )
                    .into_any_element()
            } else {
                settings_card()
                    .p(px(8.0))
                    .gap(px(6.0))
                    .children(presets.into_iter().enumerate().map(|(idx, p)| {
                        let p_id = p.id.clone();
                        let rule_type = match &p.rule {
                            ReminderRule::Once { .. } => "相对时间",
                            ReminderRule::Daily { .. } => "每日定时",
                            ReminderRule::Weekly { .. } => "每周定时",
                            ReminderRule::Monthly { .. } => "每月定时",
                            ReminderRule::Interval { .. } => "循环催办",
                        };

                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .px(px(10.0))
                            .py(px(7.0))
                            .rounded(px(6.0))
                            .bg(rgba(0xffffff05))
                            .hover(|s| s.bg(rgba(0xffffff0d)))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0x58a6ff))
                                            .child(p.label),
                                    )
                                    .child(
                                        div()
                                            .px(px(5.0))
                                            .py(px(1.0))
                                            .rounded(px(3.0))
                                            .text_xs()
                                            .text_color(rgb(0x8b949e))
                                            .bg(rgba(0xffffff08))
                                            .child(rule_type),
                                    ),
                            )
                            .child(
                                div()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .rounded(px(4.0))
                                    .cursor_pointer()
                                    .text_color(rgb(0x8b949e))
                                    .hover(|s| s.bg(rgba(0xff4d4d25)).text_color(rgb(0xff6b6b)))
                                    .id(ElementId::Name(format!("del-preset-{idx}").into()))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.data.delete_preset(&p_id);
                                        this.save(cx);
                                    }))
                                    .child(Icon::new(IconName::Delete).size(px(12.0))),
                            )
                    }))
                    .into_any_element()
            })
    }

    fn render_step_btn(
        id: &'static str,
        label: &'static str,
        on_click: impl Fn(&mut Self, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .w(px(24.0))
            .h(px(24.0))
            .flex()
            .justify_center()
            .items_center()
            .rounded(px(4.0))
            .bg(rgba(0xffffff10))
            .hover(|s| s.bg(rgba(0xffffff20)))
            .cursor_pointer()
            .id(id)
            .on_click(cx.listener(move |this, _, _, cx| {
                on_click(this, cx);
                cx.notify();
            }))
            .child(label)
    }

    fn render_type_tab(
        &self,
        idx: usize,
        id: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_sel = self.new_preset_type == idx;
        div()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .text_xs()
            .text_color(if is_sel { rgb(0xffffff) } else { rgb(0x8b949e) })
            .bg(if is_sel {
                rgba(0x38bdf835)
            } else {
                rgba(0xffffff08)
            })
            .border_1()
            .border_color(if is_sel {
                rgb(0x38bdf8)
            } else {
                rgba(0xffffff10)
            })
            .id(id)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.new_preset_type = idx;
                cx.notify();
            }))
            .child(label)
    }

    fn render_add_card(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let p_type = self.new_preset_type;
        let rel_mins = self.new_relative_mins;
        let d_hour = self.new_daily_hour;
        let d_min = self.new_daily_min;
        let int_mins = self.new_interval_mins;

        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(settings_section_header("添加新提醒预设"))
            .child(
                settings_card()
                    .p(px(14.0))
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.0))
                            .child(self.render_type_tab(0, "tab-rel", "相对时间", cx))
                            .child(self.render_type_tab(1, "tab-daily", "每日定时", cx))
                            .child(self.render_type_tab(2, "tab-int", "循环催办", cx)),
                    )
                    .child(match p_type {
                        0 => div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(div().text_xs().text_color(rgb(0x8b949e)).child("提前时间:"))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(Self::render_step_btn(
                                        "rel-m",
                                        "-",
                                        |this, _| {
                                            this.new_relative_mins =
                                                this.new_relative_mins.saturating_sub(5).max(5);
                                        },
                                        cx,
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xf0f6fc))
                                            .child(format!("{rel_mins} 分钟后")),
                                    )
                                    .child(Self::render_step_btn(
                                        "rel-p",
                                        "+",
                                        |this, _| {
                                            this.new_relative_mins =
                                                (this.new_relative_mins + 5).min(240);
                                        },
                                        cx,
                                    )),
                            )
                            .into_any_element(),
                        1 => div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x8b949e))
                                    .child("提醒时刻 (时:分):"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(Self::render_step_btn(
                                        "h-m",
                                        "-",
                                        |this, _| {
                                            this.new_daily_hour =
                                                this.new_daily_hour.saturating_sub(1);
                                        },
                                        cx,
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xf0f6fc))
                                            .child(format!("{:02}:{:02}", d_hour, d_min)),
                                    )
                                    .child(Self::render_step_btn(
                                        "h-p",
                                        "+",
                                        |this, _| {
                                            this.new_daily_hour = (this.new_daily_hour + 1) % 24;
                                        },
                                        cx,
                                    )),
                            )
                            .into_any_element(),
                        _ => div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(div().text_xs().text_color(rgb(0x8b949e)).child("催办间隔:"))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(Self::render_step_btn(
                                        "int-m",
                                        "-",
                                        |this, _| {
                                            this.new_interval_mins =
                                                this.new_interval_mins.saturating_sub(5).max(5);
                                        },
                                        cx,
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xf0f6fc))
                                            .child(format!("每 {int_mins} 分钟")),
                                    )
                                    .child(Self::render_step_btn(
                                        "int-p",
                                        "+",
                                        |this, _| {
                                            this.new_interval_mins =
                                                (this.new_interval_mins + 5).min(180);
                                        },
                                        cx,
                                    )),
                            )
                            .into_any_element(),
                    })
                    .child(
                        div()
                            .w_full()
                            .py(px(7.0))
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .bg(rgba(0x23863640))
                            .text_color(rgb(0x3fb950))
                            .border_1()
                            .border_color(rgba(0x23863680))
                            .hover(|s| s.bg(rgba(0x23863660)))
                            .id("btn-add-preset-confirm")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.add_current_preset(cx);
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("添加此预设"),
                            ),
                    ),
            )
    }
}

impl Render for TodoSettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = div()
            .flex()
            .flex_col()
            .p(px(16.0))
            .gap(px(12.0))
            .child(self.render_preset_list(cx))
            .child(self.render_add_card(cx));

        render_settings_shell("待办事项 - 设置", content)
    }
}
