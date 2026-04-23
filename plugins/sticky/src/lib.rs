use gpui::*;
use widget_core::Plugin;

pub struct StickyWidget;

impl StickyWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Render for StickyWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgba(0x050507d9)) // Abyss Black transparent
            .border_1()
            .border_color(rgb(0x3d3a39)) // Warm Charcoal
            .rounded(px(8.0))
            .child(
                // stickyContent
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .p(px(16.0))
                    .bg(rgba(0xfef3c7f2)) // warm yellow
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x78350f))
                            .child("在这里记录你的想法...\n\n双击编辑内容")
                    )
            )
    }
}

pub struct StickyWidgetPlugin;

impl Plugin for StickyWidgetPlugin {
    fn id(&self) -> &'static str {
        "sticky_widget"
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::PopUp,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::new(px(1250.0), px(50.0)),
                size(px(320.0), px(360.0)),
            ))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|_| StickyWidget::new());
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        }).unwrap().into()
    }
}

