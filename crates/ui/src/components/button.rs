use gpui::*;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Default,     // primary
    Secondary,
    Destructive,
    Outline,
    Ghost,
}

pub struct Button {
    variant: ButtonVariant,
    label: SharedString,
    icon: Option<SharedString>,
    id: ElementId,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Default,
            icon: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

impl IntoElement for Button {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let (bg_color, text_color, border_color, hover_bg) = match self.variant {
            ButtonVariant::Default => (
                rgb(0x00d992), // primary
                rgb(0x050507),
                rgba(0x00000000),
                rgba(0x00d992cc),
            ),
            ButtonVariant::Secondary => (
                rgb(0x3d3a39), // secondary
                rgb(0xf2f2f2),
                rgba(0x00000000),
                rgba(0x3d3a39cc),
            ),
            ButtonVariant::Destructive => (
                rgb(0xe81123),
                rgb(0xf2f2f2),
                rgba(0x00000000),
                rgba(0xe81123cc),
            ),
            ButtonVariant::Outline => (
                rgba(0x00000000),
                rgb(0xf2f2f2),
                rgb(0x3d3a39),
                rgba(0xffffff0a),
            ),
            ButtonVariant::Ghost => (
                rgba(0x00000000),
                rgb(0xf2f2f2),
                rgba(0x00000000),
                rgba(0xffffff0a),
            ),
        };

        let mut container = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(8.0))
            .px(px(16.0))
            .py(px(8.0))
            .rounded(px(6.0))
            .bg(bg_color)
            .text_color(text_color)
            .cursor_pointer()
            .hover(|s| s.bg(hover_bg));

        if self.variant == ButtonVariant::Outline {
            container = container.border_1().border_color(border_color);
        }

        let mut content = div().flex().items_center().gap(px(6.0));
        
        if let Some(icon) = self.icon {
            content = content.child(div().text_sm().child(icon));
        }

        content = content.child(div().text_sm().font_weight(FontWeight::MEDIUM).child(self.label));

        container.child(content).into_any_element()
    }
}
