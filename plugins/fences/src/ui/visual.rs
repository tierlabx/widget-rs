use gpui::*;
use gpui_component::IconName;

/// 拖拽排序传递的数据
#[derive(Clone, Debug)]
pub struct DraggedFenceItem {
    pub cat_idx: usize,
    pub item_idx: usize,
}

pub struct FileVisualInfo {
    pub icon_name: IconName,
    pub icon_color: Hsla,
    pub badge_text: Option<String>,
    pub badge_color: Hsla,
    pub bg_color: Hsla,
}

/// 解析文件或网址类型并返回视觉配置
pub fn resolve_file_visual(path_str: &str, is_dir: bool) -> FileVisualInfo {
    if is_dir {
        return FileVisualInfo {
            icon_name: IconName::Folder,
            icon_color: rgb(0xfbbf24).into(),
            badge_text: None,
            badge_color: rgb(0xfbbf24).into(),
            bg_color: rgba(0xfbbf2415).into(),
        };
    }

    // 网页与 URL 书签识别
    if path_str.starts_with("http://") || path_str.starts_with("https://") {
        return FileVisualInfo {
            icon_name: IconName::Globe,
            icon_color: rgb(0xa855f7).into(),
            badge_text: Some("WEB".to_string()),
            badge_color: rgb(0xa855f7).into(),
            bg_color: rgba(0xa855f718).into(),
        };
    }

    let ext = path_str.split('.').next_back().unwrap_or("").to_lowercase();

    match ext.as_str() {
        // 网页快捷方式 (.url)
        "url" => FileVisualInfo {
            icon_name: IconName::Globe,
            icon_color: rgb(0xa855f7).into(),
            badge_text: Some("URL".to_string()),
            badge_color: rgb(0xa855f7).into(),
            bg_color: rgba(0xa855f718).into(),
        },
        // 程序与可执行文件
        "exe" | "lnk" | "msi" | "bat" | "cmd" => FileVisualInfo {
            icon_name: IconName::WindowMaximize,
            icon_color: rgb(0x38bdf8).into(),
            badge_text: Some(if ext == "lnk" { "LNK" } else { "EXE" }.to_string()),
            badge_color: rgb(0x38bdf8).into(),
            bg_color: rgba(0x38bdf818).into(),
        },
        // 源代码与脚本
        "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "c" | "cpp" | "h" | "hpp" | "go" | "java"
        | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "xml" | "sql" | "sh" => {
            let label = if ext.len() <= 4 {
                ext.to_uppercase()
            } else {
                "CODE".to_string()
            };
            FileVisualInfo {
                icon_name: IconName::File,
                icon_color: rgb(0x34d399).into(),
                badge_text: Some(label),
                badge_color: rgb(0x34d399).into(),
                bg_color: rgba(0x34d39918).into(),
            }
        }
        // Word 文档
        "doc" | "docx" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0x3b82f6).into(),
            badge_text: Some("DOC".to_string()),
            badge_color: rgb(0x3b82f6).into(),
            bg_color: rgba(0x3b82f620).into(),
        },
        // PDF 文档
        "pdf" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0xef4444).into(),
            badge_text: Some("PDF".to_string()),
            badge_color: rgb(0xef4444).into(),
            bg_color: rgba(0xef444420).into(),
        },
        // Excel 表格
        "xls" | "xlsx" | "csv" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0x10b981).into(),
            badge_text: Some("XLS".to_string()),
            badge_color: rgb(0x10b981).into(),
            bg_color: rgba(0x10b98120).into(),
        },
        // PPT 幻灯片
        "ppt" | "pptx" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0xf97316).into(),
            badge_text: Some("PPT".to_string()),
            badge_color: rgb(0xf97316).into(),
            bg_color: rgba(0xf9731620).into(),
        },
        // 纯文本与 Markdown
        "md" | "txt" | "log" | "rtf" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0x86efac).into(),
            badge_text: Some(if ext == "md" { "MD" } else { "TXT" }.to_string()),
            badge_color: rgb(0x86efac).into(),
            bg_color: rgba(0x86efac18).into(),
        },
        // 压缩包
        "zip" | "rar" | "7z" | "tar" | "gz" | "iso" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0xf59e0b).into(),
            badge_text: Some(ext.to_uppercase()),
            badge_color: rgb(0xf59e0b).into(),
            bg_color: rgba(0xf59e0b20).into(),
        },
        // 图片
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg" | "ico" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0xc084fc).into(),
            badge_text: Some(ext.to_uppercase()),
            badge_color: rgb(0xc084fc).into(),
            bg_color: rgba(0xc084fc18).into(),
        },
        // 视频
        "mp4" | "mkv" | "avi" | "mov" | "flv" | "wmv" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0xe879f9).into(),
            badge_text: Some("VID".to_string()),
            badge_color: rgb(0xe879f9).into(),
            bg_color: rgba(0xe879f918).into(),
        },
        // 音频
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0xf472b6).into(),
            badge_text: Some("AUD".to_string()),
            badge_color: rgb(0xf472b6).into(),
            bg_color: rgba(0xf472b618).into(),
        },
        _ => FileVisualInfo {
            icon_name: IconName::File,
            icon_color: rgb(0x94a3b8).into(),
            badge_text: if !ext.is_empty() && ext.len() <= 4 {
                Some(ext.to_uppercase())
            } else {
                None
            },
            badge_color: rgb(0x94a3b8).into(),
            bg_color: rgba(0x94a3b818).into(),
        },
    }
}
