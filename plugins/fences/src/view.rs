use std::time::Duration;

use gpui::*;

use crate::icon_extractor::get_or_extract_icon;
use crate::model::{FenceItem, FencesData, FencesModel};
use crate::section::render_category_section;

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
                        let weak_this = cx.weak_entity();

                        render_category_section(cat_idx, cat, progress, weak_this, cx)
                    })),
            )
    }
}
