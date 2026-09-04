use std::time::Duration;

use gpui::*;

use crate::model::{FenceItem, FencesData, FencesModel};
use crate::system::icon::warm_or_fetch_icon_in_background;
use crate::ui::add_modal::{render_add_url_modal, AddUrlModalState};
use crate::ui::section::render_category_section;

pub struct FencesWidget {
    pub(crate) data: FencesData,
    /// 记录各分类栏目的手风琴展开平滑进度：0.0 (完全折叠) ~ 1.0 (完全展开)
    pub(crate) expand_progress: Vec<f32>,
    pub(crate) add_url_modal: Option<AddUrlModalState>,
}

impl FencesWidget {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = FencesModel::load(cx);
        let expand_progress = data
            .categories
            .iter()
            .map(|cat| if cat.collapsed { 0.0 } else { 1.0 })
            .collect();

        // 后台异步预热提取已有项目的原生图标与网站 Favicon（默默执行，零阻塞）
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
                    async_cx
                        .background_executor()
                        .spawn(async move {
                            for path in items_to_warm {
                                let _ = warm_or_fetch_icon_in_background(&path);
                            }
                        })
                        .await;
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
            add_url_modal: None,
        }
    }

    /// 打开 GPUI 原生添加网址弹窗
    pub fn open_add_url_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.add_url_modal = Some(AddUrlModalState::new(window, cx));
        cx.notify();
    }

    /// 关闭添加网址弹窗
    pub fn close_add_url_modal(&mut self, cx: &mut Context<Self>) {
        self.add_url_modal = None;
        cx.notify();
    }

    /// 确认添加网址书签到“程序”栏目：先关闭弹窗并立刻展现，后台默默抓取 Favicon 替换
    pub fn confirm_add_url(&mut self, url: String, name: String, cx: &mut Context<Self>) {
        // 1. 立即关闭弹窗并更新列表（0 延迟响应）
        self.add_url_modal = None;

        if let Some(cat) = self.data.categories.get_mut(0) {
            cat.collapsed = false;
            if !self.expand_progress.is_empty() {
                self.expand_progress[0] = 1.0;
            }
            cat.items.push(FenceItem {
                name,
                path: url.clone(),
                is_dir: false,
            });
            FencesModel::save(&self.data, cx);
        }
        cx.notify();

        // 2. 调度到后台线程池静默抓取 Favicon，前台协程 await，绝不阻塞主线程
        let entity_weak = cx.weak_entity();
        let app_cx: &mut App = cx;
        app_cx
            .spawn(async move |async_cx| {
                async_cx
                    .background_executor()
                    .spawn(async move {
                        let _ = warm_or_fetch_icon_in_background(&url);
                    })
                    .await;
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

    /// 触发指定分类的手风琴平滑折叠/展开动画
    pub fn toggle_category_accordion(&mut self, cat_idx: usize, cx: &mut Context<Self>) {
        if cat_idx >= self.data.categories.len() {
            return;
        }

        let cat = &mut self.data.categories[cat_idx];
        cat.collapsed = !cat.collapsed;
        let is_now_collapsed = cat.collapsed;
        let target_val = if is_now_collapsed { 0.0 } else { 1.0 };
        let start_val = self.expand_progress.get(cat_idx).copied().unwrap_or(1.0);

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

                    let is_active = async_cx.update(|cx| {
                        if let Some(entity) = entity_weak.upgrade() {
                            entity.update(cx, |this, cx| {
                                if cat_idx < this.expand_progress.len() {
                                    this.expand_progress[cat_idx] = current_p;
                                    cx.notify();
                                }
                            });
                            true
                        } else {
                            false
                        }
                    });

                    if !is_active {
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
        let mut root = div()
            .relative()
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
            // 智能拖拽归类：程序、快捷方式、网页URL归入第0栏；文件夹归入第1栏；其他归入第2栏
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
                        let display_name = if raw_name.to_lowercase().ends_with(".lnk")
                            || raw_name.to_lowercase().ends_with(".url")
                        {
                            raw_name[..raw_name.len() - 4].to_string()
                        } else {
                            raw_name
                        };

                        let ext = path
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        let is_url_file = ext == "url";
                        let actual_path = if is_url_file {
                            std::fs::read_to_string(path)
                                .ok()
                                .and_then(|content| {
                                    content.lines().find_map(|line| {
                                        let trimmed = line.trim();
                                        if trimmed.to_lowercase().starts_with("url=") {
                                            Some(trimmed[4..].trim().to_string())
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .unwrap_or(path_str.clone())
                        } else {
                            path_str.clone()
                        };

                        // 智能判断归类目标：程序、快捷方式、网页URL归入第0栏；文件夹归入第1栏；其他归入第2栏
                        let target_cat = if is_dir {
                            1 // 文件夹
                        } else if ext == "exe"
                            || ext == "lnk"
                            || is_url_file
                            || actual_path.starts_with("http://")
                            || actual_path.starts_with("https://")
                        {
                            0 // 程序与网页快捷入口
                        } else {
                            2 // 文件
                        };

                        added_paths.push(actual_path.clone());

                        if let Some(cat) = this.data.categories.get_mut(target_cat) {
                            cat.collapsed = false;
                            if target_cat < this.expand_progress.len() {
                                this.expand_progress[target_cat] = 1.0;
                            }
                            cat.items.push(FenceItem {
                                name: display_name,
                                path: actual_path,
                                is_dir,
                            });
                        }
                    }
                    FencesModel::save(&this.data, cx);
                    cx.notify();

                    // 后台线程池提取新添加文件的原生图标
                    let entity_weak = cx.weak_entity();
                    let app_cx: &mut App = cx;
                    app_cx
                        .spawn(async move |async_cx| {
                            async_cx
                                .background_executor()
                                .spawn(async move {
                                    for path in added_paths {
                                        let _ = warm_or_fetch_icon_in_background(&path);
                                    }
                                })
                                .await;
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
            );

        // 若当前处于添加网址状态，在最顶层浮层叠加渲染 GPUI 原生弹窗
        if let Some(modal) = &self.add_url_modal {
            root = root.child(render_add_url_modal(
                modal,
                |this: &mut FencesWidget, _window, cx, url, name| {
                    this.confirm_add_url(url, name, cx);
                },
                |this: &mut FencesWidget, _window, cx| {
                    this.close_add_url_modal(cx);
                },
                |this: &mut FencesWidget, cx, err| {
                    if let Some(ref mut m) = this.add_url_modal {
                        m.error_msg = err;
                        cx.notify();
                    }
                },
                cx,
            ));
        }

        root
    }
}
