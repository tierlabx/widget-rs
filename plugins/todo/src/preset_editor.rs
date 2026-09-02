use gpui::*;
use gpui_component::{Icon, IconName};
use widget_core::{settings_card, settings_section_header};

use crate::model::ReminderRule;
use crate::settings_view::TodoSettingsView;

/// 渲染提醒预设列表
pub fn render_preset_list(
    view: &TodoSettingsView,
    cx: &mut Context<TodoSettingsView>,
) -> impl IntoElement {
    let presets = view.data.reminder_presets.clone();
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

/// 渲染添加新提醒预设卡片
pub fn render_add_card(
    view: &TodoSettingsView,
    cx: &mut Context<TodoSettingsView>,
) -> impl IntoElement {
    let p_type = view.new_preset_type;
    let rel_mins = view.new_relative_mins;
    let d_hour = view.new_daily_hour;
    let d_min = view.new_daily_min;
    let int_mins = view.new_interval_mins;

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
                        .child(render_type_tab(view, 0, "tab-rel", "相对时间", cx))
                        .child(render_type_tab(view, 1, "tab-daily", "每日定时", cx))
                        .child(render_type_tab(view, 2, "tab-int", "循环催办", cx)),
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
                                .child(render_step_btn(
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
                                .child(render_step_btn(
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
                                .child(render_step_btn(
                                    "h-m",
                                    "-",
                                    |this, _| {
                                        this.new_daily_hour = this.new_daily_hour.saturating_sub(1);
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
                                .child(render_step_btn(
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
                                .child(render_step_btn(
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
                                .child(render_step_btn(
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

fn render_step_btn(
    id: &'static str,
    label: &'static str,
    on_click: impl Fn(&mut TodoSettingsView, &mut Context<TodoSettingsView>) + 'static,
    cx: &mut Context<TodoSettingsView>,
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
    view: &TodoSettingsView,
    idx: usize,
    id: &'static str,
    label: &'static str,
    cx: &mut Context<TodoSettingsView>,
) -> impl IntoElement {
    let is_sel = view.new_preset_type == idx;
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
