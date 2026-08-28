use gpui::*;
use serde::{Deserialize, Serialize};
use widget_core::AppConfig;

/// 单张便签的数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StickyNote {
    pub content: String,
    pub color_index: usize,
    #[serde(default)]
    pub images: Vec<String>,
}

impl Default for StickyNote {
    fn default() -> Self {
        Self {
            content: String::new(),
            color_index: 0,
            images: Vec::new(),
        }
    }
}

/// 所有便签的持久化数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StickyData {
    /// 所有便签列表，至少保证有一张
    pub notes: Vec<StickyNote>,
    /// 当前正在显示的便签索引
    pub current_index: usize,
}

impl Default for StickyData {
    fn default() -> Self {
        Self {
            notes: vec![StickyNote::default()],
            current_index: 0,
        }
    }
}

impl StickyData {
    /// 获取当前便签（不可变引用）
    pub fn current(&self) -> &StickyNote {
        let idx = self.current_index.min(self.notes.len().saturating_sub(1));
        &self.notes[idx]
    }

    /// 获取当前便签（可变引用）
    pub fn current_mut(&mut self) -> &mut StickyNote {
        let idx = self.current_index.min(self.notes.len().saturating_sub(1));
        &mut self.notes[idx]
    }

    /// 新建一张便签，并切换到最后一张
    pub fn new_note(&mut self) {
        let color_index = self.current().color_index; // 继承当前颜色
        self.notes.push(StickyNote {
            color_index,
            ..Default::default()
        });
        self.current_index = self.notes.len() - 1;
    }

    /// 向前翻页
    pub fn prev(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    /// 向后翻页
    pub fn next(&mut self) {
        if self.current_index + 1 < self.notes.len() {
            self.current_index += 1;
        }
    }

    /// 删除当前便签（至少保留一张）
    pub fn delete_current(&mut self) {
        if self.notes.len() > 1 {
            self.notes.remove(self.current_index);
            if self.current_index >= self.notes.len() {
                self.current_index = self.notes.len() - 1;
            }
        } else {
            // 只剩一张时清空内容
            self.notes[0] = StickyNote::default();
        }
    }
}

#[allow(dead_code)]
pub struct StickyTheme {
    pub name: &'static str,
    pub bg_hex: u32,
    pub header_hex: u32,
    pub text_hex: u32,
    pub border_hex: u32,
    pub btn_hover_hex: u32,
}

pub const STICKY_THEMES: [StickyTheme; 6] = [
    // 0: 黄色 (Yellow)
    StickyTheme {
        name: "黄色",
        bg_hex: 0xfff7d1,
        header_hex: 0xfff099,
        text_hex: 0x262626,
        border_hex: 0xf5e386,
        btn_hover_hex: 0xffe680,
    },
    // 1: 绿色 (Green)
    StickyTheme {
        name: "绿色",
        bg_hex: 0xe4f9e0,
        header_hex: 0xcbf3c5,
        text_hex: 0x1e3a1a,
        border_hex: 0xb5e9ad,
        btn_hover_hex: 0xbdf0b5,
    },
    // 2: 粉色 (Pink)
    StickyTheme {
        name: "粉色",
        bg_hex: 0xffe4e1,
        header_hex: 0xffccd2,
        text_hex: 0x4a1525,
        border_hex: 0xfcb5be,
        btn_hover_hex: 0xffbec6,
    },
    // 3: 紫色 (Purple)
    StickyTheme {
        name: "紫色",
        bg_hex: 0xebd4fa,
        header_hex: 0xdfbafb,
        text_hex: 0x38144d,
        border_hex: 0xcda0f5,
        btn_hover_hex: 0xd4a8fc,
    },
    // 4: 蓝色 (Blue)
    StickyTheme {
        name: "蓝色",
        bg_hex: 0xd0f0fd,
        header_hex: 0xb3e7fc,
        text_hex: 0x103b52,
        border_hex: 0x93dbfa,
        btn_hover_hex: 0xa1defc,
    },
    // 5: 碳黑 (Charcoal)
    StickyTheme {
        name: "碳黑",
        bg_hex: 0x282828,
        header_hex: 0x1f1f1f,
        text_hex: 0xf2f2f2,
        border_hex: 0x3d3d3d,
        btn_hover_hex: 0x333333,
    },
];

pub struct StickyModel;

impl StickyModel {
    pub fn load(cx: &mut App) -> StickyData {
        if let Some(cfg) = cx.try_global::<AppConfig>() {
            // 先尝试新格式
            if let Some(data) = cfg.get_plugin_data::<StickyData>("sticky_widget") {
                // 确保至少有一张便签
                if !data.notes.is_empty() {
                    return data;
                }
            }
            // 兼容旧格式（单便签 content 字符串）
            if let Some(content) = cfg.get_plugin_data::<String>("sticky_widget") {
                return StickyData {
                    notes: vec![StickyNote {
                        content,
                        color_index: 0,
                        images: Vec::new(),
                    }],
                    current_index: 0,
                };
            }
        }
        StickyData::default()
    }

    pub fn save(data: &StickyData, cx: &mut App) {
        cx.update_global::<AppConfig, _>(|config, _| {
            config.set_plugin_data("sticky_widget", data);
        });
        widget_core::save_config_now(cx);
    }
}
