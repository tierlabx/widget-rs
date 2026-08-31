use gpui::*;
use gpui_component::{Icon, IconName};

use crate::dialog::launch_item;
use crate::model::{FenceItem, FencesModel};
use crate::view::FencesWidget;

/// 拖拽排序传递的数据
#[derive(Clone, Debug)]
pub struct DraggedFenceItem {
    pub cat_idx: usize,
    pub item_idx: usize,
}

struct DragPreview {
    name: String,
    icon_name: IconName,
    icon_color: Hsla,
}

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .px(px(6.0))
            .py(px(4.0))
            .rounded(px(6.0))
            .bg(rgba(0x0f172ae0))
            .border_1()
            .border_color(rgb(0x38bdf8))
            .shadow_lg()
            .child(
                div()
                    .text_color(self.icon_color)
                    .child(Icon::new(self.icon_name.clone()).size(px(16.0))),
            )
            .child(
                div()
                    .text_xs()
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
    let ext = item_path
        .split('.')
        .next_back()
        .unwrap_or("")
        .to_lowercase();

    let (icon_color, icon_name) = if is_dir {
        (rgb(0xfbbf24), IconName::Folder)
    } else if ext == "exe" || ext == "lnk" {
        (rgb(0x38bdf8), IconName::WindowMaximize)
    } else if ["png", "jpg", "jpeg", "webp", "gif", "bmp"].contains(&ext.as_str()) {
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

    let preview_name = item.name.clone();
    let preview_icon_name = icon_name.clone();

    div()
        .id(ElementId::Name(
            format!("fence-card-{cat_idx}-{item_idx}").into(),
        ))
        .group("item_card")
        .relative()
        .w(px(54.0))
        .h(px(58.0))
        .rounded(px(6.0))
        .bg(rgba(0x0f172a50))
        .border_1()
        .border_color(rgba(0x38bdf818))
        .hover(|s| s.bg(rgba(0x0f172ad0)).border_color(rgba(0x38bdf850)))
        .drag_over::<DraggedFenceItem>(|s, _drag, _window, _cx| {
            s.border_color(rgb(0x38bdf8)).bg(rgba(0x38bdf830))
        })
        .on_drag(
            DraggedFenceItem { cat_idx, item_idx },
            move |_drag, _offset, _window, cx| {
                cx.new(|_| DragPreview {
                    name: preview_name.clone(),
                    icon_name: preview_icon_name.clone(),
                    icon_color: icon_color.into(),
                })
            },
        )
        .on_drop(cx.listener(move |this, drag: &DraggedFenceItem, _, cx| {
            // 拖拽排序逻辑
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
                .gap(px(2.0))
                .cursor_pointer()
                .id(ElementId::Name(
                    format!("fence-launch-{cat_idx}-{item_idx}").into(),
                ))
                .on_click(cx.listener(move |_, _, _, _| {
                    launch_item(&item_path);
                }))
                .child(
                    div()
                        .w(px(24.0))
                        .h(px(24.0))
                        .flex()
                        .justify_center()
                        .items_center()
                        .text_color(icon_color)
                        .child(Icon::new(icon_name).size(px(20.0))),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xf8fafc))
                        .text_ellipsis()
                        .text_center()
                        .max_w(px(50.0))
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
