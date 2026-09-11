use crate::js_plugin::manifest::JsWidgetManifest;
use crate::js_plugin::model::JsRenderNode;

/// 注入到脚本中的宿主环境上下文
#[derive(Debug, Clone, serde::Serialize)]
pub struct HostContext {
    pub time: String,
    pub date: String,
    pub timestamp: u64,
    pub plugin_id: String,
    pub plugin_name: String,
}

impl HostContext {
    /// 构造当前时刻的宿主上下文数据
    pub fn now(manifest: &JsWidgetManifest) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        // 获取系统当前时间戳
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();

        // 计算简易本地/UTC时间（或标准格式化）
        let sec = secs % 60;
        let min = (secs / 60) % 60;
        let hour = ((secs / 3600) + 8) % 24; // 简易东八区校正

        let time_str = format!("{:02}:{:02}:{:02}", hour, min, sec);
        let date_str = "桌面小组件".to_string();

        Self {
            time: time_str,
            date: date_str,
            timestamp: secs,
            plugin_id: manifest.id.clone(),
            plugin_name: manifest.name.clone(),
        }
    }
}

/// JS 脚本执行引擎桥接
pub struct JsEngine {
    manifest: JsWidgetManifest,
    cached_script: Option<String>,
}

impl JsEngine {
    pub fn new(manifest: JsWidgetManifest) -> Self {
        let entry_path = manifest.entry_path();
        let cached_script = std::fs::read_to_string(&entry_path).ok();
        Self {
            manifest,
            cached_script,
        }
    }

    /// 重新从磁盘加载脚本（用于热重载）
    pub fn reload_script(&mut self) -> anyhow::Result<()> {
        let entry_path = self.manifest.entry_path();
        let script = std::fs::read_to_string(&entry_path)?;
        self.cached_script = Some(script);
        Ok(())
    }

    /// 执行脚本并生成渲染节点树
    pub fn render_frame(&self) -> JsRenderNode {
        let ctx = HostContext::now(&self.manifest);

        if let Some(script) = &self.cached_script {
            // 尝试从脚本中提取或解析 JSON 模板（若脚本声明了 JSON 格式的 UI 树）
            if let Some(node) = try_eval_script(script, &ctx) {
                return node;
            }
        }

        // 默认回退渲染节点（展示基础时钟与信息）
        JsRenderNode {
            node_type: "v_flex".to_string(),
            text: None,
            action: None,
            style: crate::js_plugin::model::JsNodeStyle {
                p: Some(16.0),
                gap: Some(6.0),
                items_center: Some(true),
                justify_center: Some(true),
                rounded: Some(12.0),
                bg: Some("#121215ee".to_string()),
                border_color: Some("#27272a".to_string()),
                border_width: Some(1.0),
                ..Default::default()
            },
            children: vec![
                JsRenderNode {
                    node_type: "text".to_string(),
                    text: Some(ctx.time),
                    action: None,
                    style: crate::js_plugin::model::JsNodeStyle {
                        font_size: Some(30.0),
                        bold: Some(true),
                        color: Some("#00d992".to_string()),
                        ..Default::default()
                    },
                    children: Vec::new(),
                },
                JsRenderNode {
                    node_type: "text".to_string(),
                    text: Some(format!("{} (JS 扩展)", self.manifest.name)),
                    action: None,
                    style: crate::js_plugin::model::JsNodeStyle {
                        font_size: Some(12.0),
                        color: Some("#a1a1aa".to_string()),
                        ..Default::default()
                    },
                    children: Vec::new(),
                },
            ],
        }
    }
}

/// 快速评估或从脚本中匹配提取 UI 描述
fn try_eval_script(script: &str, ctx: &HostContext) -> Option<JsRenderNode> {
    // 若脚本包含标准 JSON 结构块，可进行宏变量替换（{{time}}, {{date}}）并反序列化
    if let (Some(start), Some(end)) = (
        script.find("/*WIDGET_UI_START*/"),
        script.find("/*WIDGET_UI_END*/"),
    ) {
        let raw_json = &script[start + 19..end].trim();
        let replaced = raw_json
            .replace("{{time}}", &ctx.time)
            .replace("{{date}}", &ctx.date)
            .replace("{{plugin_name}}", &ctx.plugin_name);
        if let Ok(node) = serde_json::from_str::<JsRenderNode>(&replaced) {
            return Some(node);
        }
    }
    None
}
