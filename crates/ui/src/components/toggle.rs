use gpui::prelude::FluentBuilder;
use gpui::*;

pub fn toggle_switch(
    id: &'static str,
    enabled: bool,
    on_toggle: impl Fn(bool, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(ElementId::Name(id.into()))
        .cursor_pointer()
        .w(px(48.0))
        .h(px(26.0))
        .rounded_full()
        .bg(if enabled {
            rgb(0x00d992)
        } else {
            rgb(0x3d3a39)
        })
        .flex()
        .items_center()
        .px(px(3.0))
        .on_click(move |_, _, cx| {
            on_toggle(!enabled, cx);
        })
        .child(
            div()
                .w(px(20.0))
                .h(px(20.0))
                .rounded_full()
                .bg(rgb(0xffffff))
                .when(enabled, |d: gpui::Div| d.ml(px(22.0))),
        )
}
