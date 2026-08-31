use std::time::Duration;

use gpui::*;
use gpui_component::{Icon, IconName};

use crate::dialog::open_add_dialog;
use crate::icon_extractor::get_or_extract_icon;
use crate::item_card::{render_item_card, DraggedFenceItem};
use crate::model::{FenceItem, FencesData, FencesModel};

pub struct FencesWidget {
    pub(crate) data: FencesData,
    /// 记录各分类栏目的手风琴展开平滑进度：0.0 (完全折叠) ~ 1.0 (完全展开)
    pub(crate) expand_progress: [f32; 3],
}

impl FencesWidget {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = FencesModel::load(cx);
        let mut expand_progress = [1.0; 3];
        for (i, cat) in data.categories.iter().enumerate().take(3) {
            expand_progress[i] = if cat.collapsed { 0.0 } else { 1.0 };
        }

        // 后台异步预热提取已有项目的 Windows 原生图标
        let items_to_warm: Vec<String> = data
            .categories
            .iter()
            .flat_map(|c| c.items.iter().map(|it| it.path.clone()))
            .collect();

        if !items_to_warm.is_empty() {
            let entity_weak = cx.weak_entity();
            let app_cx: &mut App = cx;
            app_cx
                .spawn(async move |async_cx| {
                    for path in items_to_warm {
                        let _ = get_or_extract_icon(&path);
                    }
                    let _ = async_cx.update(|cx| {
                        if let Some(entity) = entity_weak.upgrade() {
                            entity.update(cx, |_, cx| {
                                cx.notify();
                            });
                        }
                    });
                })
                .detach();
        }

        Self {
            data,
            expand_progress,
        }
    }

    /// 触发指定分类的手风琴平滑折叠/展开动画
    pub fn toggle_category_accordion(&mut self, cat_idx: usize, cx: &mut Context<Self>) {
        if cat_idx >= self.data.categories.len() {
            return;
        }

        let cat = &mut self.data.categories[cat_idx];
        cat.collapsed = !cat.collapsed;
        let is_now_collapsed = cat.collapsed;
        let target_val = if is_now_collapsed { 0.0 } else { 1.0 };
        let start_val = self.expand_progress[cat_idx];

        FencesModel::save(&self.data, cx);

        // 启动平滑插值动画（约 200ms，60fps 缓动过渡）
        let entity_weak = cx.weak_entity();
        let app_cx: &mut App = cx;
        app_cx
            .spawn(async move |async_cx| {
                let total_steps = 14;
                let step_dur = Duration::from_millis(14);

                for step in 1..=total_steps {
                    async_cx.background_executor().timer(step_dur).await;
                    let t = step as f32 / total_steps as f32;
                    // SmoothStep 缓动插值: 3t^2 - 2t^3
                    let ease_t = t * t * (3.0 - 2.0 * t);
                    let current_p = start_val + (target_val - start_val) * ease_t;

                    let update_res = async_cx.update(|cx| {
                        if let Some(entity) = entity_weak.upgrade() {
                            entity.update(cx, |this, cx| {
                                if cat_idx < this.expand_progress.len() {
                                    this.expand_progress[cat_idx] = current_p;
                                    cx.notify();
                                }
                            });
                        }
                    });

                    if update_res.is_err() {
                        break;
                    }
                }

                // 确保最后一帧精确到达目标值
                let _ = async_cx.update(|cx| {
                    if let Some(entity) = entity_weak.upgrade() {
                        entity.update(cx, |this, cx| {
                            if cat_idx < this.expand_progress.len() {
                                this.expand_progress[cat_idx] = target_val;
                                cx.notify();
                            }
                        });
                    }
                });
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
                    let mut added_paths = Vec::new();
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

                        added_paths.push(path_str.clone());

                        if let Some(cat) = this.data.categories.get_mut(target_cat) {
                            cat.collapsed = false;
                            if target_cat < this.expand_progress.len() {
                                this.expand_progress[target_cat] = 1.0;
                            }
                            cat.items.push(FenceItem {
                                name: display_name,
                                path: path_str,
                                is_dir,
                            });
                        }
                    }
                    FencesModel::save(&this.data, cx);
                    cx.notify();

                    // 后台异步提取新添加文件的原生图标
                    let entity_weak = cx.weak_entity();
                    let app_cx: &mut App = cx;
                    app_cx
                        .spawn(async move |async_cx| {
                            for path in added_paths {
                                let _ = get_or_extract_icon(&path);
                            }
                            let _ = async_cx.update(|cx| {
                                if let Some(entity) = entity_weak.upgrade() {
                                    entity.update(cx, |_, cx| {
                                        cx.notify();
                                    });
                                }
                            });
                        })
                        .detach();
                }),
            )
            // ── 主容器：纵向三栏依据手风琴进度平滑分配空间 ─────────────────
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .size_full()
                    .p(px(6.0))
                    .gap(px(6.0))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .children(categories.iter().enumerate().map(|(cat_idx, cat)| {
                        let progress = self
                            .expand_progress
                            .get(cat_idx)
                            .copied()
                            .unwrap_or(if cat.collapsed { 0.0 } else { 1.0 });

                        let is_fully_collapsed = progress <= 0.001;
                        let is_fully_expanded = progress >= 0.999;
                        let cat_name = cat.name.clone();
                        let items_count = cat.items.len();
                        let cat_items = cat.items.clone();

                        let (section_icon, section_accent) = match cat_idx {
                            0 => (IconName::WindowMaximize, rgb(0x38bdf8)), // 程序：天蓝
                            1 => (IconName::Folder, rgb(0xfbbf24)),         // 文件夹：琥珀金
                            _ => (IconName::File, rgb(0x86efac)),           // 文件与文档：清新浅绿
                        };

                        let mut section_div = div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .bg(rgba(0x00000030))
                            .rounded(px(10.0))
                            .border_1()
                            .border_color(rgba(0x38bdf818))
                            .overflow_hidden();

                        // 手风琴动态空间分配：完全展开时 flex_1 自适应，折叠或过渡期间按内容高度自适应收缩
                        if is_fully_expanded {
                            section_div = section_div.flex_1().min_h_0();
                        } else {
                            section_div = section_div.flex_shrink_0();
                        }

                        let weak_this = cx.weak_entity();

                        let mut section_div = section_div
                            // ── Section 头部栏（点击触发手风琴折叠/展开动画）───────────────
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .w_full()
                                    .px(px(8.0))
                                    .py(px(5.0))
                                    .cursor_pointer()
                                    .bg(rgba(0x00000035))
                                    .hover(|s| s.bg(rgba(0x38bdf815)))
                                    .id(ElementId::Name(format!("fence-sec-hdr-{cat_idx}").into()))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.toggle_category_accordion(cat_idx, cx);
                                    }))
                                    // 左：折叠指示箭头 + 分类图标 + 分类标题 + 计数
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(5.0))
                                            .child(
                                                div().text_color(rgba(0xffffff70)).child(
                                                    Icon::new(if progress >= 0.5 {
                                                        IconName::ChevronDown
                                                    } else {
                                                        IconName::ChevronRight
                                                    })
                                                    .size(px(12.0)),
                                                ),
                                            )
                                            .child(
                                                div()
                                                    .text_color(section_accent)
                                                    .child(Icon::new(section_icon).size(px(13.0))),
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
                                            .on_click(move |_, _, cx| {
                                                cx.stop_propagation();
                                                open_add_dialog(weak_this.clone(), cat_idx, cx);
                                            })
                                            .child("+ 添加"),
                                    ),
                            );

                        // ── 内容区（根据手风琴动画进度平滑过渡展示）─────────────
                        if !is_fully_collapsed {
                            let mut content_div = div()
                                .id(ElementId::Name(
                                    format!("fence-sec-content-{cat_idx}").into(),
                                ))
                                .w_full()
                                .overflow_hidden()
                                .opacity(progress.clamp(0.0, 1.0))
                                .drag_over::<DraggedFenceItem>(|s, _drag, _window, _cx| {
                                    s.bg(rgba(0x38bdf812))
                                })
                                .on_drop(cx.listener(
                                    move |this, drag: &DraggedFenceItem, _, cx| {
                                        if let Some(src_cat) =
                                            this.data.categories.get_mut(drag.cat_idx)
                                        {
                                            if drag.item_idx < src_cat.items.len() {
                                                let moved_item =
                                                    src_cat.items.remove(drag.item_idx);
                                                if let Some(dst_cat) =
                                                    this.data.categories.get_mut(cat_idx)
                                                {
                                                    dst_cat.items.push(moved_item);
                                                    FencesModel::save(&this.data, cx);
                                                    cx.notify();
                                                }
                                            }
                                        }
                                    },
                                ));

                            if is_fully_expanded {
                                content_div = content_div
                                    .flex_1()
                                    .min_h_0()
                                    .p(px(6.0))
                                    .overflow_y_scroll();
                            } else {
                                // 动画过渡期间：高度动态平滑插值 0 ~ 200px
                                content_div =
                                    content_div.h(px(progress * 200.0)).p(px(6.0 * progress));
                            }

                            if cat_items.is_empty() {
                                content_div = content_div.child(
                                    div()
                                        .size_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .py(px(10.0))
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
                                    div().w_full().flex().flex_wrap().gap(px(6.0)).children(
                                        cat_items.iter().enumerate().map(|(item_idx, item)| {
                                            render_item_card(item, cat_idx, item_idx, cx)
                                        }),
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
