use gpui::*;
use gpui_component::{Icon, IconName};

use crate::dialog::open_add_dialog;
use crate::item_card::{render_item_card, DraggedFenceItem};
use crate::model::{FenceCategory, FencesModel};
use crate::view::FencesWidget;

/// 渲染桌面收纳单个手风琴分类栏目
pub fn render_category_section(
    cat_idx: usize,
    cat: &FenceCategory,
    progress: f32,
    weak_this: WeakEntity<FencesWidget>,
    cx: &mut Context<FencesWidget>,
) -> impl IntoElement {
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
                        .hover(|s| s.bg(rgba(0x38bdf840)).border_color(rgba(0x38bdf870)))
                        .id(ElementId::Name(format!("fence-sec-add-{cat_idx}").into()))
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
            .drag_over::<DraggedFenceItem>(|s, _drag, _window, _cx| s.bg(rgba(0x38bdf812)))
            .on_drop(cx.listener(move |this, drag: &DraggedFenceItem, _, cx| {
                if let Some(src_cat) = this.data.categories.get_mut(drag.cat_idx) {
                    if drag.item_idx < src_cat.items.len() {
                        let moved_item = src_cat.items.remove(drag.item_idx);
                        if let Some(dst_cat) = this.data.categories.get_mut(cat_idx) {
                            dst_cat.items.push(moved_item);
                            FencesModel::save(&this.data, cx);
                            cx.notify();
                        }
                    }
                }
            }));

        if is_fully_expanded {
            content_div = content_div
                .flex_1()
                .min_h_0()
                .p(px(6.0))
                .overflow_y_scroll();
        } else {
            // 动画过渡期间：高度动态平滑插值 0 ~ 200px
            content_div = content_div.h(px(progress * 200.0)).p(px(6.0 * progress));
        }

        if cat_items.is_empty() {
            content_div = content_div.child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .py(px(10.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgba(0xffffff38))
                            .child(match cat_idx {
                                0 => "拖拽应用程序或快捷方式到此处",
                                1 => "拖拽文件夹到此处",
                                _ => "拖拽文件或文档到此处",
                            }),
                    ),
            );
        } else {
            content_div = content_div.child(
                div().w_full().flex().flex_wrap().gap(px(6.0)).children(
                    cat_items
                        .iter()
                        .enumerate()
                        .map(|(item_idx, item)| render_item_card(item, cat_idx, item_idx, cx)),
                ),
            );
        }
        section_div = section_div.child(content_div);
    }

    section_div
}
