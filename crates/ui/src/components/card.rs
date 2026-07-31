use gpui::*;

pub struct Card {
    header: Option<AnyElement>,
    content: Option<AnyElement>,
    footer: Option<AnyElement>,
    height: Option<Pixels>,
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    pub fn new() -> Self {
        Self {
            header: None,
            content: None,
            footer: None,
            height: None,
        }
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn fixed_height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }
}

impl IntoElement for Card {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let mut container = div()
            .flex_1()
            .flex()
            .flex_col()
            .p(px(16.0))
            .gap(px(12.0))
            .rounded(px(8.0))
            .bg(rgb(0x101010))
            .border_1()
            .border_color(rgb(0x3d3a39));

        if let Some(h) = self.height {
            container = container.h(h);
        }

        if let Some(h) = self.header {
            container = container.child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .w_full()
                    .child(h),
            );
        }

        if let Some(c) = self.content {
            // The content area is the key to preventing the overflow bug
            container = container.child(div().flex_1().w_full().overflow_hidden().child(c));
        }

        if let Some(f) = self.footer {
            container = container.child(div().flex().justify_end().w_full().gap(px(8.0)).child(f));
        }

        container.into_any_element()
    }
}
