use gpui_component::IconName;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// JS 小部件扩展窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsWindowConfig {
    #[serde(default = "default_width")]
    pub width: f32,
    #[serde(default = "default_height")]
    pub height: f32,
    #[serde(default, alias = "blur")]
    pub blurred: bool,
    #[serde(default)]
    pub always_on_top: bool,
}

fn default_width() -> f32 {
    280.0
}

fn default_height() -> f32 {
    140.0
}

impl Default for JsWindowConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            blurred: false,
            always_on_top: false,
        }
    }
}

/// JS 小部件清单配置（对应 widget.json / gpui-shell.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsWidgetManifest {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default = "default_entry")]
    pub entry: String,
    #[serde(default)]
    pub window: JsWindowConfig,
    #[serde(default)]
    pub has_settings: bool,
    /// 插件私有用户自定义配置
    #[serde(default)]
    pub settings: serde_json::Value,
    /// 插件根目录的绝对路径（运行时填充）
    #[serde(skip)]
    pub root_dir: PathBuf,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_author() -> String {
    "社区开发者".to_string()
}

fn default_icon() -> String {
    "Clock".to_string()
}

fn default_entry() -> String {
    "main.js".to_string()
}

impl JsWidgetManifest {
    /// 从插件目录加载 widget.json 或 gpui-shell.json
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        let manifest_path = if dir.join("widget.json").exists() {
            dir.join("widget.json")
        } else if dir.join("gpui-shell.json").exists() {
            dir.join("gpui-shell.json")
        } else {
            return Err(anyhow::anyhow!(
                "在目录 {:?} 中未找到 widget.json 或 gpui-shell.json",
                dir
            ));
        };

        let content = std::fs::read_to_string(&manifest_path)?;
        let mut manifest: Self = serde_json::from_str(&content)?;
        manifest.root_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        Ok(manifest)
    }

    /// 解析图标名称为 GPUI Component 的 IconName
    pub fn parse_icon(&self) -> IconName {
        match self.icon.to_ascii_lowercase().as_str() {
            "folder" => IconName::Folder,
            "globe" | "web" => IconName::Globe,
            "check" | "done" => IconName::Check,
            "circlecheck" => IconName::CircleCheck,
            "palette" | "theme" => IconName::Palette,
            "plus" | "add" => IconName::Plus,
            "file" | "doc" => IconName::File,
            _ => IconName::WindowMaximize,
        }
    }

    /// 获取脚本入口文件的绝对路径
    pub fn entry_path(&self) -> PathBuf {
        self.root_dir.join(&self.entry)
    }
}
