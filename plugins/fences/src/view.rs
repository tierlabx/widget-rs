use gpui::*;
use gpui_component::{Icon, IconName};

use crate::model::{FenceItem, FencesData, FencesModel};

pub struct FencesWidget {
    data: FencesData,
    scroll_handle: ScrollHandle,
}

impl FencesWidget {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = FencesModel::load(cx);
        Self {
            data,
            scroll_handle: ScrollHandle::new(),
        }
    }

    /// 打开选中的文件或文件夹
    fn launch_item(path: &str) {
        let p = path.to_string();
        std::thread::spawn(move || {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &format!("Start-Process '{}'", p)])
                .spawn();
        });
    }

    /// 打开添加文件/文件夹的对话框
    fn open_add_dialog(&mut self, target_cat: usize, cx: &mut Context<Self>) {
        let this = cx.weak_entity();
        let app_cx: &mut App = cx;

        app_cx
            .spawn(async move |async_cx| {
                let script = if target_cat == 1 {
                    "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
                     $f = New-Object System.Windows.Forms.FolderBrowserDialog; \
                     $f.Description = '选择要收纳的文件夹'; \
                     if($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ Write-Output $f.SelectedPath }"
                } else if target_cat == 0 {
                    "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
                     $f = New-Object System.Windows.Forms.OpenFileDialog; \
                     $f.Title = '选择要收纳的程序或快捷方式'; \
                     $f.Filter = '应用程序 (*.exe;*.lnk)|*.exe;*.lnk|所有文件 (*.*)|*.*'; \
                     if($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ Write-Output $f.FileName }"
                } else {
                    "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
                     $f = New-Object System.Windows.Forms.OpenFileDialog; \
                     $f.Title = '选择要收纳的文件'; \
                     $f.Filter = '所有文件 (*.*)|*.*'; \
                     if($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ Write-Output $f.FileName }"
                };

                let output = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", script])
                    .output();

                if let Ok(out) = output {
                    let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        let path = std::path::Path::new(&path_str);
                        let raw_name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path_str.clone());
                        let name = if raw_name.to_lowercase().ends_with(".lnk") {
                            raw_name[..raw_name.len() - 4].to_string()
                        } else {
                            raw_name
                        };
                        let is_dir = path.is_dir();

                        let _ = async_cx.update(|cx| {
                            let _ = this.update(cx, |this, cx| {
                                if let Some(cat) = this.data.categories.get_mut(target_cat) {
                                    cat.collapsed = false;
                                    cat.items.push(FenceItem {
                                        name,
                                        path: path_str,
                                        is_dir,
                                    });
                                    FencesModel::save(&this.data, cx);
                                    cx.notify();
                                }
                            });
                        });
                    }
                }
            })
            .detach();
    }
}

impl widget_core::WidgetContent for FencesWidget {
    fn plugin_id(&self) -> &'static str {
        "fences_widget"
    }

    fn drag_label(&self) -> &'static str {
        "拖拽移动桌面收纳"
    }
}

impl Render for FencesWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let categories = self.data.categories.clone();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .bg(rgba(0x060e1db5)) // 优雅深邃青蓝黑半透明收纳底板
            .rounded(px(14.0))
            .border_1()
            .border_color(rgba(0x38bdf825)) // 细腻的青蓝微光边框
            .overflow_hidden()
            .min_h_0()
            // 智能拖拽归类：文件夹 -> 文件夹栏；exe/lnk -> 程序栏；其他 -> 文件栏
            .on_drop(
                cx.listener(move |this, paths: &gpui::ExternalPaths, _, cx| {
                    for path in paths.paths() {
                        let path_str = path.to_string_lossy().to_string();
                        let is_dir = path.is_dir();
                        let raw_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("新建项目")
                            .to_string();
                        let display_name = if raw_name.to_lowercase().ends_with(".lnk") {
                            raw_name[..raw_name.len() - 4].to_string()
                        } else {
                            raw_name
                        };

                        let ext = path
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        let target_cat = if is_dir {
                            1 // 文件夹
                        } else if ext == "exe" || ext == "lnk" {
                            0 // 程序
                        } else {
                            2 // 文件
                        };

                        if let Some(cat) = this.data.categories.get_mut(target_cat) {
                            cat.collapsed = false;
                            cat.items.push(FenceItem {
                                name: display_name,
                                path: path_str,
                                is_dir,
                            });
                        }
                    }
                    FencesModel::save(&this.data, cx);
                    cx.notify();
                }),
            )
            // ── 滚动容器：纵向容纳三栏（程序、文件夹、文件）─────────────────
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p(px(8.0))
                    .gap(px(8.0))
                    .flex()
                    .flex_col()
                    .id("fences-scroll")
                    .track_scroll(&self.scroll_handle)
                    .overflow_y_scroll()
                    .children(categories.iter().enumerate().map(|(cat_idx, cat)| {
                        let is_collapsed = cat.collapsed;
                        let cat_name = cat.name.clone();
                        let items_count = cat.items.len();
                        let cat_items = cat.items.clone();

                        let (section_icon, section_accent) = match cat_idx {
                            0 => (IconName::WindowMaximize, rgb(0x38bdf8)), // 程序：天蓝
                            1 => (IconName::Folder, rgb(0xfbbf24)),         // 文件夹：琥珀金
                            _ => (IconName::File, rgb(0x86efac)),           // 文件：清新浅绿
                        };

                        let mut section_div = div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .bg(rgba(0x00000030))
                            .rounded(px(10.0))
                            .border_1()
                            .border_color(rgba(0x38bdf818))
                            .overflow_hidden()
                            // ── Section 头部栏（点击折叠/展开）───────────────
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .w_full()
                                    .px(px(10.0))
                                    .py(px(6.0))
                                    .cursor_pointer()
                                    .bg(rgba(0x00000035))
                                    .hover(|s| s.bg(rgba(0x38bdf815)))
                                    .id(ElementId::Name(format!("fence-sec-hdr-{cat_idx}").into()))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(c) = this.data.categories.get_mut(cat_idx) {
                                            c.collapsed = !c.collapsed;
                                            FencesModel::save(&this.data, cx);
                                            cx.notify();
                                        }
                                    }))
                                    // 左：折叠指示箭头 + 分类图标 + 分类标题 + 计数
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .child(
                                                div().text_color(rgba(0xffffff70)).child(
                                                    Icon::new(if is_collapsed {
                                                        IconName::ChevronRight
                                                    } else {
                                                        IconName::ChevronDown
                                                    })
                                                    .size(px(12.0)),
                                                ),
                                            )
                                            .child(
                                                div()
                                                    .text_color(section_accent)
                                                    .child(Icon::new(section_icon).size(px(14.0))),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0xf8fafc))
                                                    .child(cat_name),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgba(0xffffff50))
                                                    .child(format!("({})", items_count)),
                                            ),
                                    )
                                    // 右：快速添加按钮
                                    .child(
                                        div()
                                            .px(px(6.0))
                                            .py(px(1.5))
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .text_xs()
                                            .text_color(rgb(0x7dd3fc))
                                            .bg(rgba(0x38bdf820))
                                            .border_1()
                                            .border_color(rgba(0x38bdf830))
                                            .hover(|s| {
                                                s.bg(rgba(0x38bdf840))
                                                    .border_color(rgba(0x38bdf870))
                                            })
                                            .id(ElementId::Name(
                                                format!("fence-sec-add-{cat_idx}").into(),
                                            ))
                                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                                cx.stop_propagation();
                                            })
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.open_add_dialog(cat_idx, cx);
                                            }))
                                            .child("+ 添加"),
                                    ),
                            );

                        if !is_collapsed {
                            let mut content_div = div().w_full().p(px(8.0));
                            if cat_items.is_empty() {
                                content_div = content_div.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .py(px(14.0))
                                        .child(div().text_xs().text_color(rgba(0xffffff38)).child(
                                            match cat_idx {
                                                0 => "拖拽应用程序或快捷方式到此处",
                                                1 => "拖拽文件夹到此处",
                                                _ => "拖拽文件或文档到此处",
                                            },
                                        )),
                                );
                            } else {
                                content_div = content_div.child(
                                    div().flex().flex_wrap().gap(px(8.0)).children(
                                        cat_items.into_iter().enumerate().map(
                                            |(item_idx, item)| {
                                                let item_path = item.path.clone();
                                                let is_dir = item.is_dir;
                                                let ext = item_path
                                                    .split('.')
                                                    .last()
                                                    .unwrap_or("")
                                                    .to_lowercase();

                                                let (icon_color, icon_name) = if is_dir {
                                                    (rgb(0xfbbf24), IconName::Folder)
                                                } else if ext == "exe" || ext == "lnk" {
                                                    (rgb(0x38bdf8), IconName::WindowMaximize)
                                                } else if [
                                                    "png", "jpg", "jpeg", "webp", "gif", "bmp",
                                                ]
                                                .contains(&ext.as_str())
                                                {
                                                    (rgb(0xc084fc), IconName::File)
                                                } else if ["zip", "rar", "7z"]
                                                    .contains(&ext.as_str())
                                                {
                                                    (rgb(0xfb923c), IconName::File)
                                                } else if ["md", "txt"].contains(&ext.as_str()) {
                                                    (rgb(0x86efac), IconName::File)
                                                } else if ["pdf"].contains(&ext.as_str()) {
                                                    (rgb(0xf87171), IconName::File)
                                                } else if ["doc", "docx"].contains(&ext.as_str()) {
                                                    (rgb(0x67e8f9), IconName::File)
                                                } else {
                                                    (rgb(0x94a3b8), IconName::File)
                                                };

                                                div()
                                                    .relative()
                                                    .w(px(72.0))
                                                    .h(px(78.0))
                                                    .rounded(px(8.0))
                                                    .bg(rgba(0x0f172a50))
                                                    .border_1()
                                                    .border_color(rgba(0x38bdf818))
                                                    .hover(|s| {
                                                        s.bg(rgba(0x0f172ad0))
                                                            .border_color(rgba(0x38bdf850))
                                                    })
                                                    // 1. 主点击区域（点击打开/运行）
                                                    .child(
                                                        div()
                                                            .size_full()
                                                            .flex()
                                                            .flex_col()
                                                            .items_center()
                                                            .justify_center()
                                                            .p(px(4.0))
                                                            .gap(px(4.0))
                                                            .cursor_pointer()
                                                            .id(ElementId::Name(
                                                                format!(
                                                                "fence-launch-{cat_idx}-{item_idx}"
                                                            )
                                                                .into(),
                                                            ))
                                                            .on_click(cx.listener(
                                                                move |_, _, _, _| {
                                                                    Self::launch_item(&item_path);
                                                                },
                                                            ))
                                                            .child(
                                                                div()
                                                                    .w(px(30.0))
                                                                    .h(px(30.0))
                                                                    .flex()
                                                                    .justify_center()
                                                                    .items_center()
                                                                    .text_color(icon_color)
                                                                    .child(
                                                                        Icon::new(icon_name)
                                                                            .size(px(24.0)),
                                                                    ),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(FontWeight::MEDIUM)
                                                                    .text_color(rgb(0xf8fafc))
                                                                    .text_ellipsis()
                                                                    .text_center()
                                                                    .max_w(px(66.0))
                                                                    .overflow_hidden()
                                                                    .child(item.name.clone()),
                                                            ),
                                                    )
                                                    // 2. 右上角独立删除按钮（阻断事件冒泡）
                                                    .child(
                                                        div()
                                                            .absolute()
                                                            .top(px(2.0))
                                                            .right(px(2.0))
                                                            .w(px(16.0))
                                                            .h(px(16.0))
                                                            .rounded_full()
                                                            .bg(rgba(0xff4d4d35))
                                                            .border_1()
                                                            .border_color(rgba(0xff4d4d80))
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
                                                                format!(
                                                                "fence-del-{cat_idx}-{item_idx}"
                                                            )
                                                                .into(),
                                                            ))
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                |_, _, cx| {
                                                                    cx.stop_propagation();
                                                                },
                                                            )
                                                            .on_click(cx.listener(
                                                                move |this, _, _, cx| {
                                                                    cx.stop_propagation();
                                                                    if let Some(c) = this
                                                                        .data
                                                                        .categories
                                                                        .get_mut(cat_idx)
                                                                    {
                                                                        if item_idx < c.items.len()
                                                                        {
                                                                            c.items
                                                                                .remove(item_idx);
                                                                            FencesModel::save(
                                                                                &this.data, cx,
                                                                            );
                                                                            cx.notify();
                                                                        }
                                                                    }
                                                                },
                                                            ))
                                                            .child(
                                                                Icon::new(IconName::Close)
                                                                    .size(px(7.0)),
                                                            ),
                                                    )
                                            },
                                        ),
                                    ),
                                );
                            }
                            section_div = section_div.child(content_div);
                        }

                        section_div
                    })),
            )
    }
}
