use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_component::{Icon, IconName};

use crate::model::{StickyData, StickyModel, StickyNote, STICKY_THEMES};

/// 便签拖拽排序载荷
#[derive(Clone, Debug, PartialEq, Eq)]
struct DraggedNote {
    from_idx: usize,
}

/// 便签拖拽跨随预览
struct NoteDragPreview {
    text: String,
    bg_hex: u32,
    text_hex: u32,
}

impl Render for NoteDragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.0))
            .py(px(5.0))
            .bg(rgb(self.bg_hex))
            .rounded(px(6.0))
            .border_1()
            .border_color(rgb(self.text_hex))
            .shadow_lg()
            .text_xs()
            .text_color(rgb(self.text_hex))
            .child(self.text.clone())
    }
}

pub struct StickyWidget {
    input: Entity<TextareaState>,
    title_input: Entity<InputState>,
    data: StickyData,
    show_palette: bool,
    pending_input_reset: bool,
}

impl StickyWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = StickyModel::load(cx);
        let input = Self::make_textarea(window, cx, &data);
        let title_input = Self::make_title_input(window, cx, &data);
        Self {
            input,
            title_input,
            data,
            show_palette: false,
            pending_input_reset: false,
        }
    }

    fn make_textarea(
        window: &mut Window,
        cx: &mut Context<Self>,
        data: &StickyData,
    ) -> Entity<TextareaState> {
        let content = data.current().content.clone();
        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .default_value(content)
                .placeholder("在这里记录你的想法...")
        });
        cx.subscribe(
            &input,
            |this: &mut Self, input: Entity<TextareaState>, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    this.data.current_mut().content = input.read(cx).value().to_string();
                    StickyModel::save(&this.data, cx);
                }
            },
        )
        .detach();
        input
    }

    fn make_title_input(
        window: &mut Window,
        cx: &mut Context<Self>,
        data: &StickyData,
    ) -> Entity<InputState> {
        let title = data.current().title.clone();
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(title)
                .placeholder("便签标题...")
        });
        cx.subscribe(
            &input,
            |this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    this.data.current_mut().title = input.read(cx).value().to_string();
                    StickyModel::save(&this.data, cx);
                }
            },
        )
        .detach();
        input
    }
}

impl widget_core::WidgetContent for StickyWidget {
    fn plugin_id(&self) -> &'static str {
        "sticky_widget"
    }
    fn drag_label(&self) -> &'static str {
        "拖拽移动便签"
    }
}

impl Render for StickyWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 主题颜色适配
        let current_color_index = self.data.current().color_index.min(STICKY_THEMES.len() - 1);
        let is_dark_theme = STICKY_THEMES[current_color_index].is_dark;
        let theme_mut = gpui_component::Theme::global_mut(cx);
        if is_dark_theme {
            theme_mut.colors.foreground = gpui::hsla(0.0, 0.0, 0.90, 1.0);
            theme_mut.colors.muted_foreground = gpui::hsla(0.0, 0.0, 0.65, 1.0);
            theme_mut.colors.caret = gpui::hsla(0.0, 0.0, 0.92, 1.0);
            theme_mut.colors.selection = gpui::hsla(0.0, 0.0, 1.0, 0.18);
        } else {
            theme_mut.colors.foreground = gpui::hsla(0.0, 0.0, 0.12, 1.0);
            theme_mut.colors.muted_foreground = gpui::hsla(0.0, 0.0, 0.40, 1.0);
            theme_mut.colors.caret = gpui::hsla(0.0, 0.0, 0.10, 1.0);
            theme_mut.colors.selection = gpui::hsla(0.0, 0.0, 0.0, 0.15);
        }

        if self.pending_input_reset {
            self.pending_input_reset = false;
            self.input = Self::make_textarea(window, cx, &self.data);
            self.title_input = Self::make_title_input(window, cx, &self.data);
        }

        let current_note = self.data.current().clone();
        let theme = &STICKY_THEMES[current_note.color_index.min(STICKY_THEMES.len() - 1)];
        let bg_color = rgb(theme.bg_hex);
        let header_color = rgb(theme.header_hex);
        let text_color = rgb(theme.text_hex);
        let border_color = rgb(theme.border_hex);

        let current_idx = self.data.current_index;
        let show_palette = self.show_palette;
        let new_input = self.input.clone();
        let images = current_note.images.clone();

        // 半透明文字色
        let h = ((theme.text_hex >> 16) & 0xFF) as f32 / 255.0;
        let s = ((theme.text_hex >> 8) & 0xFF) as f32 / 255.0;
        let l = (theme.text_hex & 0xFF) as f32 / 255.0;
        let text_muted = hsla(h / 360.0, s, l, 0.55);
        let text_faint = hsla(h / 360.0, s, l, 0.28);

        // 克隆 notes 用于侧边栏渲染
        let notes: Vec<StickyNote> = self.data.notes.clone();
        let note_count = notes.len();

        div()
            .relative()
            .size_full()
            .on_drop(cx.listener(|this, paths: &gpui::ExternalPaths, _, cx| {
                for path in paths.paths() {
                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if ["png", "jpg", "jpeg", "webp", "gif", "bmp"].contains(&ext.as_str()) {
                        this.data
                            .current_mut()
                            .images
                            .push(path.to_string_lossy().to_string());
                        StickyModel::save(&this.data, cx);
                        cx.notify();
                    }
                }
            }))
            // ── 浮动便签 Tab 侧边栏：absolute left:0 w:104，右对齐贴齐内容面板，允许向左溢出 ──
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(0.0))
                    .bottom(px(0.0))
                    .w(px(104.0)) // 104px 浮动空间基准，长标签向左延伸不撞窗口左边界，右边缘紧贴内容面板
                    .flex()
                    .flex_col()
                    .items_end() // Tab 右对齐，紧贴内容面板左边缘，文字长时向左朝外超出
                    .gap(px(3.0))
                    .pt(px(10.0))
                    .pb(px(6.0))
                    // 不加 overflow_hidden，允许长标签朝左边超出，不裁剪文字
                    // 各便签 Tab
                    .children(notes.iter().enumerate().map(|(idx, note)| {
                        let is_active = idx == current_idx;
                        let note_theme =
                            &STICKY_THEMES[note.color_index.min(STICKY_THEMES.len() - 1)];
                        let tab_label = note.tab_label(idx);

                        // 激活色：使用该便签自身的主题色
                        let active_bg = rgb(note_theme.header_hex);
                        let active_border = rgb(note_theme.border_hex);
                        let active_text: Hsla = if note_theme.is_dark {
                            rgb(0xf0f0f0).into()
                        } else {
                            rgb(note_theme.text_hex).into()
                        };

                        div()
                            .relative()
                            .h(px(32.0))
                            .rounded_l(px(7.0))
                            .px(px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(3.0))
                            .whitespace_nowrap() // 防止换行
                            .cursor_grab()
                            .text_size(px(10.5))
                            .font_weight(if is_active {
                                FontWeight::BOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .text_color(if is_active {
                                active_text
                            } else {
                                rgba(0x000000bb).into()
                            })
                            .bg(if is_active {
                                active_bg
                            } else {
                                rgba(0xffffff35)
                            })
                            .border_1()
                            .border_color(if is_active {
                                active_border
                            } else {
                                rgba(0x00000015)
                            })
                            .hover(|s| {
                                s.bg(if is_active {
                                    active_bg
                                } else {
                                    rgba(0xffffff60)
                                })
                            })
                            // id 必须在 on_drag 之前，将 Div 转为 Stateful<Div>
                            .id(ElementId::Name(format!("sticky-tab-{idx}").into()))
                            // 拖拽悬停高亮
                            .drag_over::<DraggedNote>(|s, _, _, _| {
                                s.border_color(rgb(note_theme.border_hex))
                                    .bg(rgba(note_theme.bg_hex | 0x99))
                            })
                            // 放下：触发排序
                            .on_drop(cx.listener(move |this, drag: &DraggedNote, _, cx| {
                                this.data.reorder_note(drag.from_idx, idx);
                                this.pending_input_reset = true;
                                StickyModel::save(&this.data, cx);
                                cx.notify();
                            }))
                            // 开始拖拽：生成预览
                            .on_drag(DraggedNote { from_idx: idx }, {
                                let preview_text = note.tab_label(idx);
                                let preview_bg = note_theme.bg_hex;
                                let preview_text_hex = note_theme.text_hex;
                                move |_, _, _, cx| {
                                    cx.new(|_| NoteDragPreview {
                                        text: preview_text.clone(),
                                        bg_hex: preview_bg,
                                        text_hex: preview_text_hex,
                                    })
                                }
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.data.switch_to(idx);
                                this.pending_input_reset = true;
                                StickyModel::save(&this.data, cx);
                                cx.notify();
                            }))
                            // 激活时左侧竖条指示
                            .when(is_active, |d: Stateful<Div>| {
                                d.child(
                                    div()
                                        .absolute()
                                        .left(px(0.0))
                                        .top(px(6.0))
                                        .bottom(px(6.0))
                                        .w(px(3.0))
                                        .rounded_r(px(2.0))
                                        .bg(rgb(note_theme.text_hex)),
                                )
                            })
                            .child(div().text_align(TextAlign::Center).child(tab_label))
                    }))
                    // 底部新建便签按钮
                    .child(
                        div()
                            .h(px(26.0))
                            .rounded_l(px(6.0))
                            .px(px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .text_color(rgba(0x00000060))
                            .bg(rgba(0xffffff20))
                            .border_1()
                            .border_color(rgba(0x00000015))
                            .hover(|s| s.bg(rgba(0xffffff60)).text_color(rgba(0x000000cc)))
                            .id("sticky-new-tab")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.data.new_note();
                                this.pending_input_reset = true;
                                StickyModel::save(&this.data, cx);
                                cx.notify();
                            }))
                            .child(Icon::new(IconName::Plus).size(px(11.0))),
                    )
                    // 便签数量指示
                    .when(note_count > 1, |d: Div| {
                        d.child(
                            div()
                                .w_full()
                                .flex()
                                .justify_center()
                                .text_size(px(9.0))
                                .text_color(rgba(0x00000055))
                                .child(format!("{}/{}", current_idx + 1, note_count)),
                        )
                    }),
            )
            // ── 主内容区：absolute 铺满，left=104→right:0 ───────────────────────
            .child(
                div()
                    .absolute()
                    .left(px(104.0))
                    .right(px(0.0))
                    .top(px(0.0))
                    .bottom(px(0.0))
                    .flex()
                    .flex_col()
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .overflow_hidden()
                    .min_h_0()
                    // ── Header ────────────────────────────────────────────
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .px(px(8.0))
                            .py(px(4.0))
                            .bg(header_color)
                            .border_b_1()
                            .border_color(border_color)
                            .flex_shrink_0()
                            // 左：标题输入框
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(text_muted)
                                    .child(
                                        Input::new(&self.title_input)
                                            .appearance(false)
                                            .bordered(false),
                                    ),
                            )
                            // 右：调色板 + 删除
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(2.0))
                                    .flex_shrink_0()
                                    // 调色板
                                    .child(
                                        div()
                                            .w(px(22.0))
                                            .h(px(22.0))
                                            .flex()
                                            .justify_center()
                                            .items_center()
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .text_color(text_faint)
                                            .hover(|s| {
                                                s.bg(rgba(0x00000018)).text_color(text_color)
                                            })
                                            .id("sticky-palette")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.show_palette = !this.show_palette;
                                                cx.notify();
                                            }))
                                            .child(Icon::new(IconName::Palette).size(px(12.0))),
                                    )
                                    // 删除
                                    .child(
                                        div()
                                            .w(px(22.0))
                                            .h(px(22.0))
                                            .flex()
                                            .justify_center()
                                            .items_center()
                                            .rounded(px(4.0))
                                            .cursor_pointer()
                                            .text_color(text_faint)
                                            .hover(|s| {
                                                s.bg(rgba(0xff000018)).text_color(rgb(0xff4444))
                                            })
                                            .id("sticky-del")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.data.delete_current();
                                                this.pending_input_reset = true;
                                                StickyModel::save(&this.data, cx);
                                                cx.notify();
                                            }))
                                            .child(Icon::new(IconName::Delete).size(px(12.0))),
                                    ),
                            ),
                    )
                    // ── 调色板 ────────────────────────────────────────────
                    .when(show_palette, |d| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(5.0))
                                .px(px(10.0))
                                .py(px(6.0))
                                .bg(header_color)
                                .border_b_1()
                                .border_color(border_color)
                                .flex_shrink_0()
                                .children(STICKY_THEMES.iter().enumerate().map(|(i, t)| {
                                    let is_active = i == current_note.color_index;
                                    div()
                                        .w(px(18.0))
                                        .h(px(18.0))
                                        .rounded_full()
                                        .cursor_pointer()
                                        .bg(rgb(t.bg_hex))
                                        .border_2()
                                        .border_color(if is_active {
                                            rgb(0x444444)
                                        } else {
                                            rgb(t.border_hex)
                                        })
                                        .id(ElementId::Name(format!("sticky-color-{i}").into()))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.data.current_mut().color_index = i;
                                            this.show_palette = false;
                                            StickyModel::save(&this.data, cx);
                                            cx.notify();
                                        }))
                                })),
                        )
                    })
                    // ── 图片预览区 ────────────────────────────────────────
                    .when(!images.is_empty(), |d| {
                        d.child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(4.0))
                                .p(px(6.0))
                                .border_b_1()
                                .border_color(border_color)
                                .flex_shrink_0()
                                .children(images.iter().enumerate().map(|(img_idx, path)| {
                                    div()
                                        .relative()
                                        .w(px(60.0))
                                        .h(px(60.0))
                                        .rounded(px(4.0))
                                        .overflow_hidden()
                                        .border_1()
                                        .border_color(border_color)
                                        .child(
                                            img(std::path::PathBuf::from(path))
                                                .w_full()
                                                .h_full()
                                                .object_fit(ObjectFit::Cover),
                                        )
                                        .child(
                                            div()
                                                .absolute()
                                                .top(px(2.0))
                                                .right(px(2.0))
                                                .w(px(14.0))
                                                .h(px(14.0))
                                                .rounded_full()
                                                .bg(rgba(0x00000088))
                                                .text_color(rgb(0xffffff))
                                                .flex()
                                                .justify_center()
                                                .items_center()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgba(0xff000088)))
                                                .id(ElementId::Name(
                                                    format!("sticky-del-img-{img_idx}").into(),
                                                ))
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    let imgs = &mut this.data.current_mut().images;
                                                    if img_idx < imgs.len() {
                                                        imgs.remove(img_idx);
                                                    }
                                                    StickyModel::save(&this.data, cx);
                                                    cx.notify();
                                                }))
                                                .child(Icon::new(IconName::Close).size(px(7.0))),
                                        )
                                })),
                        )
                    })
                    // ── 文本编辑区 ────────────────────────────────────────
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .min_h_0()
                            .overflow_hidden()
                            .text_sm()
                            .text_color(text_color)
                            .child(
                                Textarea::new(&new_input)
                                    .appearance(false)
                                    .bordered(false)
                                    .size_full(),
                            ),
                    ),
            )
    }
}
