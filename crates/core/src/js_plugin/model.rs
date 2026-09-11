use gpui::*;
use serde::{Deserialize, Serialize};

/// JS 小组件返回的样式描述
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsNodeStyle {
    #[serde(default)]
    pub p: Option<f32>,
    #[serde(default)]
    pub px: Option<f32>,
    #[serde(default)]
    pub py: Option<f32>,
    #[serde(default)]
    pub gap: Option<f32>,
    #[serde(default)]
    pub font_size: Option<f32>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub bg: Option<String>,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default)]
    pub border_width: Option<f32>,
    #[serde(default)]
    pub rounded: Option<f32>,
    #[serde(default)]
    pub items_center: Option<bool>,
    #[serde(default)]
    pub justify_center: Option<bool>,
    #[serde(default)]
    pub justify_between: Option<bool>,
    #[serde(default)]
    pub flex_grow: Option<bool>,
}

/// JS 小组件渲染节点描述（树状结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsRenderNode {
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub style: JsNodeStyle,
    #[serde(default)]
    pub children: Vec<JsRenderNode>,
}

impl JsRenderNode {
    /// 创建文本节点
    pub fn new_text(text: impl Into<String>, font_size: f32, color: impl Into<String>) -> Self {
        Self {
            node_type: "text".to_string(),
            text: Some(text.into()),
            action: None,
            style: JsNodeStyle {
                font_size: Some(font_size),
                color: Some(color.into()),
                ..Default::default()
            },
            children: Vec::new(),
        }
    }

    /// 将描述节点实时重放为 GPUI 原生元素
    pub fn into_element(self) -> AnyElement {
        match self.node_type.as_str() {
            "text" => {
                let txt = self.text.unwrap_or_default();
                let mut el = div().child(txt);
                if let Some(sz) = self.style.font_size {
                    el = el.text_size(px(sz));
                }
                if let Some(true) = self.style.bold {
                    el = el.font_weight(FontWeight::BOLD);
                }
                if let Some(col) = self.style.color {
                    if let Some(parsed) = parse_hex_color(&col) {
                        el = el.text_color(parsed);
                    }
                }
                el.into_any_element()
            }
            "button" => {
                let label = self.text.unwrap_or_else(|| "确定".to_string());
                let mut el = div()
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(label);

                if let Some(p) = self.style.p {
                    el = el.p(px(p));
                } else {
                    let px_val = self.style.px.unwrap_or(12.0);
                    let py_val = self.style.py.unwrap_or(6.0);
                    el = el.px(px(px_val)).py(px(py_val));
                }
                if let Some(r) = self.style.rounded {
                    el = el.rounded(px(r));
                } else {
                    el = el.rounded(px(6.0));
                }
                if let Some(bg_str) = self.style.bg {
                    if let Some(c) = parse_hex_color(&bg_str) {
                        el = el.bg(c);
                    }
                } else {
                    el = el.bg(rgb(0x27272a));
                }
                if let Some(col) = self.style.color {
                    if let Some(c) = parse_hex_color(&col) {
                        el = el.text_color(c);
                    }
                }
                el.into_any_element()
            }
            "divider" => div()
                .w_full()
                .h(px(1.0))
                .bg(rgb(0x27272a))
                .my(px(8.0))
                .into_any_element(),
            _ => {
                // 默认为 flex 容器（v_flex 或 h_flex 或普通 div）
                let is_horizontal = self.node_type == "h_flex" || self.node_type == "row";
                let mut container = div().flex();
                if is_horizontal {
                    container = container.flex_row();
                } else {
                    container = container.flex_col();
                }

                if let Some(p) = self.style.p {
                    container = container.p(px(p));
                }
                if let Some(px_val) = self.style.px {
                    container = container.px(px(px_val));
                }
                if let Some(py_val) = self.style.py {
                    container = container.py(px(py_val));
                }
                if let Some(g) = self.style.gap {
                    container = container.gap(px(g));
                }
                if let Some(r) = self.style.rounded {
                    container = container.rounded(px(r));
                }
                if let Some(true) = self.style.items_center {
                    container = container.items_center();
                }
                if let Some(true) = self.style.justify_center {
                    container = container.justify_center();
                }
                if let Some(true) = self.style.justify_between {
                    container = container.justify_between();
                }
                if let Some(bg_str) = self.style.bg {
                    if let Some(c) = parse_hex_color(&bg_str) {
                        container = container.bg(c);
                    }
                }
                if let Some(bc_str) = self.style.border_color {
                    if let Some(c) = parse_hex_color(&bc_str) {
                        let bw = self.style.border_width.unwrap_or(1.0);
                        container = container.border(px(bw)).border_color(c);
                    }
                }

                let child_elements: Vec<AnyElement> = self
                    .children
                    .into_iter()
                    .map(|child| child.into_element())
                    .collect();

                container.children(child_elements).into_any_element()
            }
        }
    }
}

/// 解析常见 16 进制颜色格式（例如 "#00d992" 或 "#27272a" 或 "rgb(...)"）
fn parse_hex_color(hex_str: &str) -> Option<Hsla> {
    let s = hex_str.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(rgb((r as u32) << 16 | (g as u32) << 8 | (b as u32)).into())
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(rgba((r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | (a as u32)).into())
    } else {
        None
    }
}
