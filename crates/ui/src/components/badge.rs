use gpui::*;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
}

pub struct Badge {
    variant: BadgeVariant,
    label: SharedString,
    show_dot: bool,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
            show_dot: false,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn show_dot(mut self, show: bool) -> Self {
        self.show_dot = show;
        self
    }
}

impl IntoElement for Badge {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let (bg_color, text_color, border_color, dot_color) = match self.variant {
            BadgeVariant::Default => (
                rgba(0x00d9921a), // tinted primary bg
                rgb(0x00d992),
                rgba(0x00000000),
                rgb(0x00d992),
            ),
            BadgeVariant::Secondary => (
                rgba(0x8b949e33), // tinted secondary bg
                rgb(0x8b949e),
                rgba(0x00000000),
                rgb(0x8b949e),
            ),
            BadgeVariant::Destructive => (
                rgba(0xe8112333),
                rgb(0xe81123),
                rgba(0x00000000),
                rgb(0xe81123),
            ),
            BadgeVariant::Outline => (
                rgba(0x00000000),
                rgb(0xf2f2f2),
                rgb(0x3d3a39),
                rgb(0xf2f2f2),
            ),
        };

        let mut container = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(10.0))
            .py(px(4.0))
            .rounded_full()
            .bg(bg_color);

        if self.variant == BadgeVariant::Outline {
            container = container.border_1().border_color(border_color);
        }

        if self.show_dot {
            container = container.child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(dot_color));
        }

        container
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(self.label),
            )
            .into_any_element()
    }
}
