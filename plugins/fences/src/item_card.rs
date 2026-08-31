use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{Icon, IconName};

use crate::dialog::launch_item;
use crate::icon_extractor::get_or_extract_icon;
use crate::model::{FenceItem, FencesModel};
use crate::view::FencesWidget;

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

/// 解析文件类型并返回视觉配置
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

    let ext = path_str.split('.').next_back().unwrap_or("").to_lowercase();

    match ext.as_str() {
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
            bg_color: rgba(0x94a3b815).into(),
        },
    }
}

struct DragPreview {
    name: String,
    icon_name: IconName,
    icon_color: Hsla,
    native_icon_path: Option<String>,
}

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .py(px(5.0))
            .rounded(px(6.0))
            .bg(rgba(0x0f172af0))
            .border_1()
            .border_color(rgb(0x38bdf8))
            .shadow_lg();

        if let Some(ref path) = self.native_icon_path {
            let img_source = gpui::ImageSource::Resource(gpui::Resource::Path(Arc::from(
                std::path::Path::new(path),
            )));
            row = row.child(
                div()
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded(px(3.0))
                    .overflow_hidden()
                    .child(img(img_source).size_full()),
            );
        } else {
            row = row.child(
                div()
                    .text_color(self.icon_color)
                    .child(Icon::new(self.icon_name.clone()).size(px(16.0))),
            );
        }

        row.child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xf8fafc))
                .child(self.name.clone()),
        )
    }
}

/// 渲染单个桌面收纳项卡片
pub fn render_item_card(
    item: &FenceItem,
    cat_idx: usize,
    item_idx: usize,
    cx: &mut Context<FencesWidget>,
) -> impl IntoElement {
    let item_path = item.path.clone();
    let is_dir = item.is_dir;
    let visual = resolve_file_visual(&item_path, is_dir);

    // 优先尝试获取或提取 Windows 原生高清程序/文件图标
    let native_icon = if !is_dir {
        get_or_extract_icon(&item_path)
    } else {
        None
    };

    let preview_name = item.name.clone();
    let preview_icon_name = visual.icon_name.clone();
    let preview_icon_color = visual.icon_color;
    let preview_native_icon = native_icon.clone();

    div()
        .id(ElementId::Name(
            format!("fence-card-{cat_idx}-{item_idx}").into(),
        ))
        .group("item_card")
        .relative()
        .w(px(58.0))
        .h(px(66.0))
        .rounded(px(8.0))
        .bg(visual.bg_color)
        .border_1()
        .border_color(rgba(0x38bdf825))
        .hover(|s| s.bg(rgba(0x0f172ae8)).border_color(rgba(0x38bdf880)))
        .drag_over::<DraggedFenceItem>(|s, _drag, _window, _cx| {
            s.border_color(rgb(0x38bdf8)).bg(rgba(0x38bdf840))
        })
        .on_drag(
            DraggedFenceItem { cat_idx, item_idx },
            move |_drag, _offset, _window, cx| {
                cx.new(|_| DragPreview {
                    name: preview_name.clone(),
                    icon_name: preview_icon_name.clone(),
                    icon_color: preview_icon_color,
                    native_icon_path: preview_native_icon.clone(),
                })
            },
        )
        .on_drop(cx.listener(move |this, drag: &DraggedFenceItem, _, cx| {
            if drag.cat_idx == cat_idx && drag.item_idx == item_idx {
                return;
            }
            if let Some(src_cat) = this.data.categories.get_mut(drag.cat_idx) {
                if drag.item_idx < src_cat.items.len() {
                    let moved_item = src_cat.items.remove(drag.item_idx);
                    if let Some(dst_cat) = this.data.categories.get_mut(cat_idx) {
                        let insert_idx = item_idx.min(dst_cat.items.len());
                        dst_cat.items.insert(insert_idx, moved_item);
                        FencesModel::save(&this.data, cx);
                        cx.notify();
                    }
                }
            }
        }))
        // 1. 主点击区域（点击打开/运行）
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .p(px(2.0))
                .gap(px(3.0))
                .cursor_pointer()
                .id(ElementId::Name(
                    format!("fence-launch-{cat_idx}-{item_idx}").into(),
                ))
                .on_click(cx.listener(move |_, _, _, _| {
                    launch_item(&item_path);
                }))
                // 图标展示区域：原生图标 或 精致矢量图标+Badge
                .child(
                    div()
                        .relative()
                        .w(px(30.0))
                        .h(px(30.0))
                        .flex()
                        .justify_center()
                        .items_center()
                        .map(|parent| {
                            if let Some(ref icon_file) = native_icon {
                                let img_source = gpui::ImageSource::Resource(gpui::Resource::Path(
                                    Arc::from(std::path::Path::new(icon_file)),
                                ));
                                parent.child(
                                    div()
                                        .w(px(28.0))
                                        .h(px(28.0))
                                        .rounded(px(4.0))
                                        .overflow_hidden()
                                        .child(img(img_source).size_full()),
                                )
                            } else {
                                let mut icon_box = parent
                                    .text_color(visual.icon_color)
                                    .child(Icon::new(visual.icon_name).size(px(22.0)));

                                if let Some(badge) = visual.badge_text {
                                    icon_box = icon_box.child(
                                        div()
                                            .absolute()
                                            .bottom(px(-2.0))
                                            .right(px(-4.0))
                                            .px(px(2.5))
                                            .py(px(0.5))
                                            .rounded(px(3.0))
                                            .bg(rgba(0x0f172af0))
                                            .border_1()
                                            .border_color(visual.badge_color)
                                            .text_color(visual.badge_color)
                                            .font_weight(FontWeight::BOLD)
                                            .text_size(px(7.5))
                                            .line_height(px(8.0))
                                            .child(badge),
                                    );
                                }
                                icon_box
                            }
                        }),
                )
                // 文件/程序名称
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xf8fafc))
                        .text_ellipsis()
                        .text_center()
                        .max_w(px(54.0))
                        .overflow_hidden()
                        .child(item.name.clone()),
                ),
        )
        // 2. 右上角独立删除按钮（阻断事件冒泡，鼠标移入卡片时才显示）
        .child(
            div()
                .invisible()
                .group_hover("item_card", |s| s.visible())
                .absolute()
                .top(px(1.0))
                .right(px(1.0))
                .w(px(14.0))
                .h(px(14.0))
                .rounded_full()
                .bg(rgba(0xff4d4d45))
                .border_1()
                .border_color(rgba(0xff4d4d90))
                .text_color(rgb(0xffa0a0))
                .flex()
                .justify_center()
                .items_center()
                .cursor_pointer()
                .hover(|s| {
                    s.bg(rgb(0xef4444))
                        .border_color(rgb(0xffffff))
                        .text_color(rgb(0xffffff))
                })
                .id(ElementId::Name(
                    format!("fence-del-{cat_idx}-{item_idx}").into(),
                ))
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    if let Some(c) = this.data.categories.get_mut(cat_idx) {
                        if item_idx < c.items.len() {
                            c.items.remove(item_idx);
                            FencesModel::save(&this.data, cx);
                            cx.notify();
                        }
                    }
                }))
                .child(Icon::new(IconName::Close).size(px(6.0))),
        )
}
