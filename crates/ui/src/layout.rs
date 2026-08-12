use gpui::*;

pub fn page_header(title: &str, subtitle: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_3xl()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xf2f2f2))
                .child(title.to_string()),
        )
        .child(
            div()
                .text_base()
                .text_color(rgb(0xb8b3b0))
                .child(subtitle.to_string()),
        )
}

pub fn section_title(title: &str) -> impl IntoElement {
    div()
        .text_base()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(0x8b949e))
        .child(title.to_string())
}

pub fn settings_card() -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .rounded(px(10.0))
        .bg(rgb(0x101010))
        .border_1()
        .border_color(rgb(0x3d3a39))
}

pub fn settings_row(has_border_bottom: bool) -> Div {
    let row = div()
        .flex()
        .justify_between()
        .items_center()
        .w_full()
        .p(px(20.0));

    if has_border_bottom {
        row.border_b_1().border_color(rgb(0x3d3a39))
    } else {
        row
    }
}
