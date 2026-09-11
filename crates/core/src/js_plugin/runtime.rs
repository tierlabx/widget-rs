use crate::js_plugin::manifest::JsWidgetManifest;
use crate::js_plugin::model::JsRenderNode;

/// 注入到脚本中的宿主环境上下文
#[derive(Debug, Clone, serde::Serialize)]
pub struct HostContext {
    pub time: String,
    pub hours_minutes: String,
    pub seconds: String,
    pub date: String,
    pub timestamp: u64,
    pub plugin_id: String,
    pub plugin_name: String,
}

impl HostContext {
    /// 构造当前时刻的宿主上下文数据
    pub fn now(manifest: &JsWidgetManifest) -> Self {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::System::SystemInformation::GetLocalTime;
            let mut st = std::mem::zeroed();
            GetLocalTime(&mut st);

            let week_days = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
            let week_str = week_days.get(st.wDayOfWeek as usize).unwrap_or(&"周五");

            let time_str = format!("{:02}:{:02}:{:02}", st.wHour, st.wMinute, st.wSecond);
            let hm_str = format!("{:02}:{:02}", st.wHour, st.wMinute);
            let sec_str = format!("{:02}", st.wSecond);
            let date_str = format!("{:02}/{:02} {}", st.wMonth, st.wDay, week_str);

            Self {
                time: time_str,
                hours_minutes: hm_str,
                seconds: sec_str,
                date: date_str,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                plugin_id: manifest.id.clone(),
                plugin_name: manifest.name.clone(),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self {
                time: "12:00:00".to_string(),
                hours_minutes: "12:00".to_string(),
                seconds: "00".to_string(),
                date: "01/01 周五".to_string(),
                timestamp: 0,
                plugin_id: manifest.id.clone(),
                plugin_name: manifest.name.clone(),
            }
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
            if let Some(node) = try_eval_script(script, &ctx) {
                return node;
            }
        }

        // 默认现代亚克力半透明毛玻璃灵动岛胶囊
        JsRenderNode {
            node_type: "h_flex".to_string(),
            text: None,
            action: None,
            style: crate::js_plugin::model::JsNodeStyle {
                w_full: Some(true),
                h_full: Some(true),
                px: Some(16.0),
                py: Some(12.0),
                gap: Some(16.0),
                items_center: Some(true),
                justify_between: Some(true),
                rounded: Some(16.0),
                bg: Some("#0f172a65".to_string()),
                border_color: Some("#ffffff18".to_string()),
                border_width: Some(1.0),
                ..Default::default()
            },
            children: vec![
                // 左侧大字号时分
                JsRenderNode {
                    node_type: "text".to_string(),
                    text: Some(ctx.hours_minutes),
                    action: None,
                    style: crate::js_plugin::model::JsNodeStyle {
                        font_size: Some(40.0),
                        bold: Some(true),
                        color: Some("#f8fafc".to_string()),
                        ..Default::default()
                    },
                    children: Vec::new(),
                },
                // 右侧：秒数胶囊与日期
                JsRenderNode {
                    node_type: "v_flex".to_string(),
                    text: None,
                    action: None,
                    style: crate::js_plugin::model::JsNodeStyle {
                        gap: Some(6.0),
                        justify_center: Some(true),
                        ..Default::default()
                    },
                    children: vec![
                        // 秒针跳动芯片
                        JsRenderNode {
                            node_type: "h_flex".to_string(),
                            text: None,
                            action: None,
                            style: crate::js_plugin::model::JsNodeStyle {
                                px: Some(8.0),
                                py: Some(2.0),
                                rounded: Some(6.0),
                                bg: Some("#00d99222".to_string()),
                                border_color: Some("#00d99244".to_string()),
                                border_width: Some(1.0),
                                items_center: Some(true),
                                justify_center: Some(true),
                                ..Default::default()
                            },
                            children: vec![JsRenderNode {
                                node_type: "text".to_string(),
                                text: Some(format!("{}s", ctx.seconds)),
                                action: None,
                                style: crate::js_plugin::model::JsNodeStyle {
                                    font_size: Some(11.0),
                                    bold: Some(true),
                                    color: Some("#00d992".to_string()),
                                    ..Default::default()
                                },
                                children: Vec::new(),
                            }],
                        },
                        // 日期星期
                        JsRenderNode {
                            node_type: "text".to_string(),
                            text: Some(ctx.date),
                            action: None,
                            style: crate::js_plugin::model::JsNodeStyle {
                                font_size: Some(11.0),
                                color: Some("#94a3b8".to_string()),
                                ..Default::default()
                            },
                            children: Vec::new(),
                        },
                    ],
                },
            ],
        }
    }
}

/// 快速评估或从脚本中匹配提取 UI 描述
fn try_eval_script(script: &str, ctx: &HostContext) -> Option<JsRenderNode> {
    if let (Some(start), Some(end)) = (
        script.find("/*WIDGET_UI_START*/"),
        script.find("/*WIDGET_UI_END*/"),
    ) {
        let raw_json = &script[start + 19..end].trim();
        let replaced = raw_json
            .replace("{{hours_minutes}}", &ctx.hours_minutes)
            .replace("{{seconds}}", &ctx.seconds)
            .replace("{{time}}", &ctx.time)
            .replace("{{date}}", &ctx.date)
            .replace("{{plugin_name}}", &ctx.plugin_name);
        if let Ok(node) = serde_json::from_str::<JsRenderNode>(&replaced) {
            return Some(node);
        }
    }
    None
}
