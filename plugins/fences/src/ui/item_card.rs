use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::tooltip::Tooltip;
use gpui_component::{Icon, IconName};

use crate::model::{FenceItem, FencesModel};
use crate::system::dialog::launch_item;
use crate::system::icon::get_or_extract_icon;
use crate::ui::view::FencesWidget;
use crate::ui::visual::{resolve_file_visual, DraggedFenceItem};

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
                    .w(px(22.0))
                    .h(px(22.0))
                    .rounded(px(5.0))
                    .bg(rgba(0xffffff22))
                    .border_1()
                    .border_color(rgba(0xffffff30))
                    .p(px(1.5))
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

/// 渲染桌面收纳单个文件/程序卡片
pub fn render_item_card(
    item: &FenceItem,
    cat_idx: usize,
    item_idx: usize,
    cx: &mut Context<FencesWidget>,
) -> impl IntoElement {
    let item_path = item.path.clone();
    let is_dir = item.is_dir;
    let visual = resolve_file_visual(&item_path, is_dir);

    // 内存级高速读取 Windows 原生图标或 Favicon 缓存（纳秒级命中，零磁盘 I/O）
    let native_icon = if !is_dir {
        get_or_extract_icon(&item_path)
    } else {
        None
    };

    let preview_name = item.name.clone();
    let preview_icon_name = visual.icon_name.clone();
    let preview_icon_color = visual.icon_color;
    let preview_native_icon = native_icon.clone();

    let full_name = item.name.clone();

    div()
        .id(ElementId::Name(
            format!("fence-card-{cat_idx}-{item_idx}").into(),
        ))
        .group("item_card")
        .relative()
        .w(px(60.0))
        .h(px(68.0))
        .rounded(px(8.0))
        .bg(visual.bg_color)
        .border_1()
        .border_color(rgba(0x38bdf825))
        .tooltip(move |_window, cx| cx.new(|_cx| Tooltip::new(full_name.clone())).into())
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
        // ── 卡片点击启动区 ──────────────────────────────────────────
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
                // 图标展示区域：包裹微透明高质感底衬，彻底解决暗色/全黑 Favicon 看不清的问题
                .child(
                    div()
                        .relative()
                        .w(px(32.0))
                        .h(px(32.0))
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
                                        .w(px(30.0))
                                        .h(px(30.0))
                                        .rounded(px(6.0))
                                        .bg(rgba(0xffffff20)) // 柔和微光底板，强化对比度
                                        .border_1()
                                        .border_color(rgba(0xffffff28))
                                        .p(px(2.0))
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
                // 项目名称文本
                .child(
                    div()
                        .w_full()
                        .px(px(2.0))
                        .text_center()
                        .text_size(px(10.0))
                        .line_height(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xe2e8f0))
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(item.name.clone()),
                ),
        )
        // ── 移除按钮（右上角红色微徽章，悬停出现）────────────────
        .child(
            div()
                .absolute()
                .top(px(-4.0))
                .right(px(-4.0))
                .w(px(14.0))
                .h(px(14.0))
                .rounded(px(7.0))
                .bg(rgb(0xef4444))
                .border_1()
                .border_color(rgb(0xffffff))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .opacity(0.0)
                .group_hover("item_card", |s| s.opacity(1.0))
                .id(ElementId::Name(
                    format!("fence-del-{cat_idx}-{item_idx}").into(),
                ))
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    if let Some(cat) = this.data.categories.get_mut(cat_idx) {
                        if item_idx < cat.items.len() {
                            cat.items.remove(item_idx);
                            FencesModel::save(&this.data, cx);
                            cx.notify();
                        }
                    }
                }))
                .child(
                    div()
                        .text_color(rgb(0xffffff))
                        .child(Icon::new(IconName::Close).size(px(9.0))),
                ),
        )
}
