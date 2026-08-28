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
    fn open_add_dialog(&mut self, is_folder: bool, cx: &mut Context<Self>) {
        let this = cx.weak_entity();
        let app_cx: &mut App = cx;
        let active_cat = self.data.active_category;

        app_cx
            .spawn(async move |async_cx| {
                let script = if is_folder {
                    "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
                     $f = New-Object System.Windows.Forms.FolderBrowserDialog; \
                     $f.Description = '选择要收纳的文件夹'; \
                     if($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ Write-Output $f.SelectedPath }"
                } else {
                    "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
                     $f = New-Object System.Windows.Forms.OpenFileDialog; \
                     $f.Title = '选择要收纳的文件或程序'; \
                     $f.Filter = '所有文件 (*.*)|*.*|应用程序 (*.exe;*.lnk)|*.exe;*.lnk'; \
                     if($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ Write-Output $f.FileName }"
                };

                let output = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", script])
                    .output();

                if let Ok(out) = output {
                    let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        let path = std::path::Path::new(&path_str);
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path_str.clone());
                        let is_dir = path.is_dir();

                        let _ = async_cx.update(|cx| {
                            let _ = this.update(cx, |this, cx| {
                                if let Some(cat) = this.data.categories.get_mut(active_cat) {
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
        let active_cat_idx = self
            .data
            .active_category
            .min(self.data.categories.len().saturating_sub(1));
        let categories = self.data.categories.clone();
        let items = categories
            .get(active_cat_idx)
            .map(|c| c.items.clone())
            .unwrap_or_default();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .p(px(6.0))
            .gap(px(6.0))
            .overflow_hidden()
            .min_h_0()
            // ── 顶部：独立悬浮分类胶囊导航 + 添加按钮 ────────────────────
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px(px(10.0))
                    .py(px(7.0))
                    .bg(rgba(0x0f172ae8)) // 高质感深海蓝黑悬浮胶囊栏
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(rgba(0x38bdf825))
                    .flex_shrink_0()
                    // 左：分类胶囊标签
                    .child(div().flex().items_center().gap(px(4.0)).children(
                        categories.iter().enumerate().map(|(idx, cat)| {
                            let is_active = idx == active_cat_idx;
                            div()
                                .px(px(9.0))
                                .py(px(3.0))
                                .rounded_full()
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(if is_active {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .bg(if is_active {
                                    rgba(0x38bdf835)
                                } else {
                                    rgba(0x00000000)
                                })
                                .text_color(if is_active {
                                    rgb(0x38bdf8)
                                } else {
                                    rgba(0xffffff88)
                                })
                                .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0xffffff)))
                                .id(ElementId::Name(format!("fence-cat-{idx}").into()))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.data.active_category = idx;
                                    FencesModel::save(&this.data, cx);
                                    cx.notify();
                                }))
                                .child(cat.name.clone())
                        }),
                    ))
                    // 右：添加按钮（统一高对比度天蓝）
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .px(px(8.0))
                                    .py(px(2.5))
                                    .rounded(px(5.0))
                                    .cursor_pointer()
                                    .text_xs()
                                    .text_color(rgb(0x7dd3fc))
                                    .bg(rgba(0x38bdf820))
                                    .border_1()
                                    .border_color(rgba(0x38bdf830))
                                    .hover(|s| {
                                        s.bg(rgba(0x38bdf835)).border_color(rgba(0x38bdf860))
                                    })
                                    .id("fence-add-file")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.open_add_dialog(false, cx);
                                    }))
                                    .child("+ 文件"),
                            )
                            .child(
                                div()
                                    .px(px(8.0))
                                    .py(px(2.5))
                                    .rounded(px(5.0))
                                    .cursor_pointer()
                                    .text_xs()
                                    .text_color(rgb(0xbae6fd))
                                    .bg(rgba(0x38bdf820))
                                    .border_1()
                                    .border_color(rgba(0x38bdf830))
                                    .hover(|s| {
                                        s.bg(rgba(0x38bdf835)).border_color(rgba(0x38bdf860))
                                    })
                                    .id("fence-add-dir")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.open_add_dialog(true, cx);
                                    }))
                                    .child("+ 文件夹"),
                            ),
                    ),
            )
            // ── 图标网格内容区 ────────────────────────────────────────────
            .child({
                let items_empty = items.is_empty();
                let mut scroll_div = div()
                    .flex_1()
                    .w_full()
                    .p(px(8.0))
                    .id("fences-scroll")
                    .track_scroll(&self.scroll_handle)
                    .overflow_y_scroll();

                if items_empty {
                    // 空状态提示
                    scroll_div = scroll_div.child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .h_full()
                            .pt(px(30.0))
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_color(rgba(0xffffff20))
                                    .child(Icon::new(IconName::Folder).size(px(36.0))),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgba(0xffffff40))
                                    .child("点击右上角 + 文件 或 + 文件夹 添加快捷方式"),
                            ),
                    );
                }

                scroll_div.child(div().flex().flex_wrap().gap(px(10.0)).children(
                    items.into_iter().enumerate().map(|(item_idx, item)| {
                        let item_path = item.path.clone();
                        let is_dir = item.is_dir;
                        let ext = item_path.split('.').last().unwrap_or("").to_lowercase();

                        // 根据文件类型决定图标颜色
                        let (icon_color, icon_name) = if is_dir {
                            (rgb(0xfbbf24), IconName::Folder)
                        } else if ext == "exe" || ext == "lnk" {
                            (rgb(0x38bdf8), IconName::WindowMaximize)
                        } else if ["png", "jpg", "jpeg", "webp", "gif", "bmp"]
                            .contains(&ext.as_str())
                        {
                            (rgb(0xc084fc), IconName::File)
                        } else if ["zip", "rar", "7z"].contains(&ext.as_str()) {
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

                        // 卡片：直接悬浮于壁纸之上，hover 时呈现深色微光浮岛卡片
                        div()
                            .relative()
                            .w(px(80.0))
                            .h(px(88.0))
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .p(px(6.0))
                            .gap(px(8.0))
                            .rounded(px(10.0))
                            .cursor_pointer()
                            .bg(rgba(0x00000000))
                            .border_1()
                            .border_color(rgba(0x00000000))
                            .hover(|s| s.bg(rgba(0x0f172ad0)).border_color(rgba(0x38bdf840)))
                            .id(ElementId::Name(format!("fence-item-{item_idx}").into()))
                            .on_click(move |_, _, _| {
                                Self::launch_item(&item_path);
                            })
                            // 纯色大图标
                            .child(
                                div()
                                    .w(px(36.0))
                                    .h(px(36.0))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .text_color(icon_color)
                                    .child(Icon::new(icon_name).size(px(28.0))),
                            )
                            // 文件名（清晰高对比度白字）
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xf8fafc))
                                    .text_ellipsis()
                                    .text_center()
                                    .max_w(px(74.0))
                                    .overflow_hidden()
                                    .child(item.name.clone()),
                            )
                            // 删除角标：默认透明，自身 hover 时显现
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .right_0()
                                    .w(px(16.0))
                                    .h(px(16.0))
                                    .rounded_full()
                                    .bg(rgba(0x00000000))
                                    // 默认文字色完全透明——视觉上不存在
                                    .text_color(rgba(0xffffff00))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .cursor_pointer()
                                    // hover 到角标本身时显现
                                    .hover(|s| s.bg(rgba(0xff4d4dcc)).text_color(rgb(0xffffff)))
                                    .id(ElementId::Name(format!("fence-del-{item_idx}").into()))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(cat) =
                                            this.data.categories.get_mut(active_cat_idx)
                                        {
                                            if item_idx < cat.items.len() {
                                                cat.items.remove(item_idx);
                                                FencesModel::save(&this.data, cx);
                                                cx.notify();
                                            }
                                        }
                                    }))
                                    .child(Icon::new(IconName::Close).size(px(7.0))),
                            )
                    }),
                ))
            })
    }
}
