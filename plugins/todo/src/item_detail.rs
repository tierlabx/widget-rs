use gpui::*;

use crate::model::{ReminderPreset, ReminderRule, TodoItem, TodoTag, GANTT_COLORS};

/// 渲染待办条目的展开详情设置面板
#[allow(clippy::too_many_arguments)]
pub fn render_item_detail<V: 'static>(
    idx: usize,
    item: &TodoItem,
    all_tags: &[TodoTag],
    reminder_presets: &[ReminderPreset],
    on_change_tag: impl Fn(&mut V, &mut Window, &mut Context<V>, usize, String) + 'static + Clone,
    on_set_reminder: impl Fn(&mut V, &mut Window, &mut Context<V>, usize, Option<ReminderRule>)
        + 'static
        + Clone,
    on_set_color: impl Fn(&mut V, &mut Window, &mut Context<V>, usize, usize) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let curr_tag_id = item.tag_id.clone();
    let curr_reminder = item.reminder.clone();
    let color_idx = item.gantt_color % GANTT_COLORS.len();
    let created_time = item
        .created_at
        .clone()
        .unwrap_or_else(|| "今天".to_string());

    div()
        .flex()
        .flex_col()
        .w_full()
        .px(px(8.0))
        .py(px(6.0))
        .gap(px(6.0))
        .bg(rgba(0x00000035))
        .border_t_1()
        .border_color(rgba(0xffffff10))
        // 1. 所属分类选择
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(div().text_xs().text_color(rgba(0xffffff60)).child("分类:"))
                .children(all_tags.iter().map(|tag| {
                    let is_curr = curr_tag_id == tag.id;
                    let tag_id_clone = tag.id.clone();
                    let tag_color = &GANTT_COLORS[tag.gantt_color % GANTT_COLORS.len()];
                    let on_tag = on_change_tag.clone();
                    div()
                        .px(px(5.0))
                        .py(px(1.0))
                        .rounded(px(3.0))
                        .cursor_pointer()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if is_curr {
                            tag_color.contrast_text()
                        } else {
                            rgb(tag_color.hex).into()
                        })
                        .bg(if is_curr {
                            rgb(tag_color.hex)
                        } else {
                            rgba(tag_color.bg_alpha_hex)
                        })
                        .border_1()
                        .border_color(if is_curr {
                            rgb(tag_color.hex)
                        } else {
                            rgba(tag_color.hex | 0x45)
                        })
                        .hover(|s| s.opacity(0.85))
                        .id(ElementId::Name(format!("todo-tag-{idx}-{}", tag.id).into()))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            on_tag(this, window, cx, idx, tag_id_clone.clone());
                        }))
                        .child(tag.name.clone())
                })),
        )
        // 2. 提醒设置
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba(0xffffff60))
                        .child("提醒设置:"),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(3.0))
                        .child({
                            let is_none = curr_reminder.is_none();
                            let on_rem = on_set_reminder.clone();
                            div()
                                .px(px(5.0))
                                .py(px(1.5))
                                .rounded(px(3.0))
                                .cursor_pointer()
                                .text_xs()
                                .text_color(if is_none {
                                    rgb(0xffffff)
                                } else {
                                    rgba(0xffffff60)
                                })
                                .bg(if is_none {
                                    rgba(0x38bdf840)
                                } else {
                                    rgba(0xffffff10)
                                })
                                .id(ElementId::Name(format!("todo-rem-none-{idx}").into()))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    on_rem(this, window, cx, idx, None);
                                }))
                                .child("无")
                        })
                        .children(reminder_presets.iter().enumerate().map(|(p_idx, preset)| {
                            let on_rem = on_set_reminder.clone();
                            let preset_clone = preset.clone();
                            let preset_id = preset.id.clone();
                            let label = preset.label.clone();
                            let color_hex = match p_idx % 4 {
                                0 => (0x38bdf8, 0x38bdf818, 0x38bdf835),
                                1 => (0x34d399, 0x34d39918, 0x34d39935),
                                2 => (0xa78bfa, 0xa78bfa18, 0xa78bfa35),
                                _ => (0xfb923c, 0xfb923c20, 0xfb923c40),
                            };

                            div()
                                .px(px(5.0))
                                .py(px(1.5))
                                .rounded(px(3.0))
                                .cursor_pointer()
                                .text_xs()
                                .text_color(rgb(color_hex.0))
                                .bg(rgba(color_hex.1))
                                .hover(move |s| s.bg(rgba(color_hex.2)))
                                .id(ElementId::Name(
                                    format!("todo-rem-{preset_id}-{idx}").into(),
                                ))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    let rule = preset_clone.to_rule();
                                    on_rem(this, window, cx, idx, Some(rule));
                                }))
                                .child(label)
                        })),
                ),
        )
        // 3. 甘特色系选择与创建时间
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgba(0xffffff60))
                                .child("甘特色:"),
                        )
                        .children(GANTT_COLORS.iter().enumerate().map(|(g_idx, g_color)| {
                            let is_curr = g_idx == color_idx;
                            let on_col = on_set_color.clone();
                            div()
                                .w(px(12.0))
                                .h(px(12.0))
                                .rounded_full()
                                .cursor_pointer()
                                .bg(rgb(g_color.hex))
                                .border_2()
                                .border_color(if is_curr {
                                    rgb(0xffffff)
                                } else {
                                    rgba(0x00000000)
                                })
                                .hover(|s| s.border_color(rgb(0xffffff)))
                                .id(ElementId::Name(format!("todo-gantt-{idx}-{g_idx}").into()))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    on_col(this, window, cx, idx, g_idx);
                                }))
                        })),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba(0xffffff40))
                        .child(created_time),
                ),
        )
}
