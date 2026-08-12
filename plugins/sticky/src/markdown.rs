use gpui::*;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub fn render_markdown(text: &str) -> impl IntoElement {
    let mut elements: Vec<AnyElement> = Vec::new();

    // Very simple block-level markdown renderer for Sticky Note
    let mut current_text = String::new();
    let mut current_header = 0;
    let mut is_quote = false;

    let parser = Parser::new(text);

    for event in parser {
        match event {
            Event::Text(t) => {
                current_text.push_str(&t);
            }
            Event::Code(t) => {
                current_text.push_str(&format!("`{}`", t));
            }
            Event::Start(Tag::Strong) => {}
            Event::End(TagEnd::Strong) => {}
            Event::Start(Tag::Emphasis) => {}
            Event::End(TagEnd::Emphasis) => {}
            Event::Start(Tag::Heading { level, .. }) => {
                current_header = level as u8;
            }
            Event::End(TagEnd::Heading(_)) => {
                let mut d = div().w_full().mb(px(8.0)).child(
                    div()
                        .text_color(rgb(0x3d2000))
                        .font_weight(FontWeight::BOLD)
                        .child(current_text.clone()),
                );
                match current_header {
                    1 => d = d.text_xl(),
                    2 => d = d.text_lg(),
                    _ => d = d.text_base(),
                }
                elements.push(d.into_any_element());
                current_text.clear();
                current_header = 0;
            }
            Event::Start(Tag::BlockQuote(_)) => is_quote = true,
            Event::End(TagEnd::BlockQuote(_)) => is_quote = false,
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if !current_text.is_empty() {
                    let mut d = div()
                        .w_full()
                        .mb(px(4.0))
                        .text_sm()
                        .text_color(rgb(0x3d2000))
                        .child(current_text.clone());
                    if is_quote {
                        d = d
                            .border_l_4()
                            .border_color(rgba(0xf59e0b80))
                            .pl(px(8.0))
                            .text_color(rgb(0x78350f));
                    }
                    elements.push(d.into_any_element());
                    current_text.clear();
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                let url = dest_url.to_string();
                if url.starts_with("lmdb://") {
                    let key = url.replace("lmdb://", "");
                    if let Some(db) = crate::db::global_db() {
                        if let Some(bytes) = db.get_image(&key) {
                            // GPUI 暂不支持直接传 bytes 给 img()
                            // 作为替代方案，我们可以将图像写入 temp_dir 并返回路径
                            let temp_dir = std::env::temp_dir().join("widget-rs-images");
                            let _ = std::fs::create_dir_all(&temp_dir);
                            let temp_file = temp_dir.join(format!("{}.png", key));
                            if !temp_file.exists() {
                                let _ = std::fs::write(&temp_file, bytes);
                            }
                            if let Some(path_str) = temp_file.to_str() {
                                let image_path = format!("file:///{}", path_str.replace("\\", "/"));
                                elements.push(
                                    div()
                                        .w_full()
                                        .mb(px(8.0))
                                        .child(
                                            img(SharedString::from(image_path))
                                                .w_full()
                                                .rounded(px(4.0)),
                                        )
                                        .into_any_element(),
                                );
                            }
                        }
                    }
                } else {
                    elements.push(
                        div()
                            .w_full()
                            .mb(px(8.0))
                            .child(img(SharedString::from(url)).w_full().rounded(px(4.0)))
                            .into_any_element(),
                    );
                }
            }
            Event::HardBreak | Event::SoftBreak => {
                current_text.push('\n');
            }
            _ => {}
        }
    }

    // 如果结尾还有未提交的文本
    if !current_text.is_empty() {
        elements.push(
            div()
                .w_full()
                .text_sm()
                .text_color(rgb(0x3d2000))
                .child(current_text.clone())
                .into_any_element(),
        );
    }

    div()
        .id("sticky-md-scroll")
        .w_full()
        .h_full()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .children(elements)
}
