use gpui::*;

use crate::tips::current_tip;

/// 渲染“健康小贴士”与“周期轮次”展开卡片
pub fn render_details_card(
    mini_taken: u32,
    mini_total: u32,
    dot_color: Rgba,
    is_warning: bool,
    is_paused: bool,
) -> impl IntoElement {
    let tip = current_tip();

    div()
        .flex()
        .flex_col()
        .w_full()
        .pt(px(6.0))
        .mt(px(4.0))
        .gap(px(6.0))
        .border_t_1()
        .border_color(rgba(0xffffff15))
        // 💡 建议提示条
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .p(px(6.0))
                .rounded(px(6.0))
                .bg(rgba(0x00000030))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x38bdf8))
                        .font_weight(FontWeight::MEDIUM)
                        .child("💡 建议:"),
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .text_color(rgba(0xffffff85))
                        .child(tip),
                ),
        )
        // 周期与状态标签
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .px(px(2.0))
                .child(div().text_xs().text_color(rgba(0xffffff50)).child(format!(
                    "微休周期: 第 {}/{} 轮",
                    mini_taken + 1,
                    mini_total
                )))
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(1.5))
                        .rounded_full()
                        .text_xs()
                        .text_color(dot_color)
                        .bg(rgba(0xffffff10))
                        .child(if is_warning {
                            "预警中"
                        } else if is_paused {
                            "计时暂停"
                        } else {
                            "高效专注"
                        }),
                ),
        )
}
