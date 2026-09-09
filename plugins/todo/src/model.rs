use gpui::*;
use serde::{Deserialize, Serialize};
use widget_core::AppConfig;

/// 经典项目甘特色系定义
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct GanttColor {
    pub name: &'static str,
    pub hex: u32,
    pub bg_alpha_hex: u32,
}

impl GanttColor {
    /// 计算以该色为纯色背景时的最佳文本前景色（浅底配深字，深底配白字）
    pub fn contrast_text(&self) -> Hsla {
        let r = ((self.hex >> 16) & 0xff) as f32;
        let g = ((self.hex >> 8) & 0xff) as f32;
        let b = (self.hex & 0xff) as f32;
        let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
        if luminance > 150.0 {
            rgb(0x0f172a).into()
        } else {
            rgb(0xffffff).into()
        }
    }
}

pub const GANTT_COLORS: &[GanttColor] = &[
    GanttColor {
        name: "标准蓝",
        hex: 0x38bdf8,
        bg_alpha_hex: 0x38bdf830,
    }, // 天空蓝
    GanttColor {
        name: "进行绿",
        hex: 0x34d399,
        bg_alpha_hex: 0x34d39930,
    }, // 翡翠绿
    GanttColor {
        name: "核心紫",
        hex: 0xa78bfa,
        bg_alpha_hex: 0xa78bfa30,
    }, // 薰衣紫
    GanttColor {
        name: "关注橙",
        hex: 0xfb923c,
        bg_alpha_hex: 0xfb923c30,
    }, // 活力橙
    GanttColor {
        name: "紧急红",
        hex: 0xf87171,
        bg_alpha_hex: 0xf8717130,
    }, // 珊瑚红
    GanttColor {
        name: "规划金",
        hex: 0xfacc15,
        bg_alpha_hex: 0xfacc1530,
    }, // 日光金
];

pub use crate::reminder::*;

/// 分类标签
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoTag {
    pub id: String,
    pub name: String,
    pub gantt_color: usize,
}

/// 单条待办任务
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub done: bool,
    #[serde(default)]
    pub tag_id: String,
    #[serde(default)]
    pub gantt_color: usize,
    #[serde(default)]
    pub reminder: Option<ReminderRule>,
    #[serde(default)]
    pub last_reminded_at: Option<u64>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// 待办数据总集（包含任务、分类标签与提醒预设）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoData {
    pub active_tag_id: String,
    pub tags: Vec<TodoTag>,
    pub items: Vec<TodoItem>,
    #[serde(default = "default_reminder_presets")]
    pub reminder_presets: Vec<ReminderPreset>,
}

impl TodoData {
    /// 拖拽排序：将指定 ID 的待办项移动到目标 ID 项的位置
    /// - 若向下拖动（源索引 < 目标索引）：移动到目标项之后
    /// - 若向上拖动（源索引 > 目标索引）：移动到目标项之前
    pub fn reorder_item(&mut self, source_id: &str, target_id: &str) -> bool {
        if source_id == target_id {
            return false;
        }
        let src_pos = match self.items.iter().position(|it| it.id == source_id) {
            Some(p) => p,
            None => return false,
        };
        let target_pos = match self.items.iter().position(|it| it.id == target_id) {
            Some(p) => p,
            None => return false,
        };

        let item = self.items.remove(src_pos);
        let new_target_pos = self
            .items
            .iter()
            .position(|it| it.id == target_id)
            .unwrap_or(self.items.len());

        if src_pos < target_pos {
            self.items.insert(new_target_pos + 1, item);
        } else {
            self.items.insert(new_target_pos, item);
        }
        true
    }

    /// 新增分类标签，返回新标签 ID
    pub fn add_tag(&mut self, name: String, gantt_color: usize) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let new_id = format!("tag-{}", now);
        self.tags.push(TodoTag {
            id: new_id.clone(),
            name,
            gantt_color,
        });
        new_id
    }

    /// 更新分类标签名称与颜色
    pub fn update_tag(&mut self, tag_id: &str, name: String, gantt_color: usize) -> bool {
        if let Some(tag) = self.tags.iter_mut().find(|t| t.id == tag_id) {
            tag.name = name;
            tag.gantt_color = gantt_color;
            true
        } else {
            false
        }
    }

    /// 安全删除分类标签，并将该标签下的任务迁移到首个可用标签
    pub fn delete_tag_and_migrate(&mut self, tag_id: &str) -> bool {
        if self.tags.len() <= 1 {
            return false;
        }

        if let Some(pos) = self.tags.iter().position(|t| t.id == tag_id) {
            self.tags.remove(pos);

            let fallback_tag_id = self
                .tags
                .first()
                .map(|t| t.id.clone())
                .unwrap_or_else(|| "work".to_string());

            // 迁移该分类下的条目
            for item in &mut self.items {
                if item.tag_id == tag_id {
                    item.tag_id = fallback_tag_id.clone();
                }
            }

            // 若当前处于被删除分类的视图下，重置为全部
            if self.active_tag_id == tag_id {
                self.active_tag_id = "all".to_string();
            }

            true
        } else {
            false
        }
    }

    /// 新增提醒预设
    pub fn add_preset(&mut self, label: String, rule: ReminderRule) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let new_id = format!("preset-{}", now);
        self.reminder_presets.push(ReminderPreset {
            id: new_id.clone(),
            label,
            rule,
        });
        new_id
    }

    /// 更新提醒预设
    #[allow(dead_code)]
    pub fn update_preset(&mut self, preset_id: &str, label: String, rule: ReminderRule) -> bool {
        if let Some(p) = self.reminder_presets.iter_mut().find(|p| p.id == preset_id) {
            p.label = label;
            p.rule = rule;
            true
        } else {
            false
        }
    }

    /// 删除提醒预设
    pub fn delete_preset(&mut self, preset_id: &str) -> bool {
        if let Some(pos) = self.reminder_presets.iter().position(|p| p.id == preset_id) {
            self.reminder_presets.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Default for TodoData {
    fn default() -> Self {
        Self {
            active_tag_id: "all".to_string(),
            tags: vec![
                TodoTag {
                    id: "work".to_string(),
                    name: "工作".to_string(),
                    gantt_color: 0,
                },
                TodoTag {
                    id: "study".to_string(),
                    name: "学习".to_string(),
                    gantt_color: 1,
                },
                TodoTag {
                    id: "life".to_string(),
                    name: "生活".to_string(),
                    gantt_color: 2,
                },
                TodoTag {
                    id: "shopping".to_string(),
                    name: "购物".to_string(),
                    gantt_color: 3,
                },
            ],
            items: vec![],
            reminder_presets: default_reminder_presets(),
        }
    }
}

pub struct TodoModel;

impl TodoModel {
    pub fn load(cx: &mut App) -> TodoData {
        if let Some(cfg) = cx.try_global::<AppConfig>() {
            // 尝试读取新版 TodoData
            if let Some(data) = cfg.get_plugin_data::<TodoData>("todo_widget") {
                return data;
            }
            // 尝试迁移旧版 Vec<TodoItem>
            if let Some(old_items) = cfg.get_plugin_data::<Vec<TodoItem>>("todo_widget") {
                return TodoData {
                    items: old_items,
                    ..Default::default()
                };
            }
        }
        TodoData::default()
    }

    pub fn save(data: &TodoData, cx: &mut App) {
        cx.update_global::<AppConfig, _>(|cfg, _| {
            cfg.set_plugin_data("todo_widget", data);
        });
        widget_core::save_config_now(cx);
    }
}
