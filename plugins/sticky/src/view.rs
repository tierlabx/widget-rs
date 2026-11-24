use gpui::*;
use gpui_component::input::{InputEvent, InputState, TextareaState};

use crate::content_panel::render_content_panel;
use crate::model::{StickyData, StickyModel, STICKY_THEMES};
use crate::sidebar::render_sidebar;

pub struct StickyWidget {
    pub(crate) input: Entity<TextareaState>,
    _input_sub: gpui::Subscription,
    pub(crate) title_input: Entity<InputState>,
    _title_input_sub: gpui::Subscription,
    pub(crate) data: StickyData,
    pub(crate) show_palette: bool,
    tab_scroll_handle: ScrollHandle,
    save_debounce_task: Option<gpui::Task<()>>,
}

impl StickyWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = StickyModel::load(cx);
        let init_content = data.current().content.clone();
        let init_title = data.current().title.clone();

        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .default_value(init_content)
                .placeholder("在这里记录你的想法...")
        });
        let input_sub = cx.subscribe(
            &input,
            |this: &mut Self, input: Entity<TextareaState>, event: &InputEvent, cx| match event {
                InputEvent::Change => {
                    let idx = this.data.current_index;
                    let prev_label = if this.data.current().title.is_empty() {
                        Some(this.data.current().tab_label(idx))
                    } else {
                        None
                    };
                    this.data.current_mut().content = input.read(cx).value().to_string();
                    this.schedule_debounced_save(cx);
                    if let Some(prev) = prev_label {
                        if this.data.current().tab_label(idx) != prev {
                            cx.notify();
                        }
                    }
                }
                InputEvent::Blur => {
                    this.flush_pending_save(cx);
                }
                _ => {}
            },
        );

        let title_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(init_title)
                .placeholder("便签标题...")
        });
        let title_input_sub = cx.subscribe(
            &title_input,
            |this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| match event {
                InputEvent::Change => {
                    this.data.current_mut().title = input.read(cx).value().to_string();
                    this.schedule_debounced_save(cx);
                    cx.notify();
                }
                InputEvent::Blur => {
                    this.flush_pending_save(cx);
                }
                _ => {}
            },
        );

        Self {
            input,
            _input_sub: input_sub,
            title_input,
            _title_input_sub: title_input_sub,
            data,
            show_palette: false,
            tab_scroll_handle: ScrollHandle::new(),
            save_debounce_task: None,
        }
    }

    /// 复用既有输入实体同步当前便签的标题与正文，避免反复销毁重建 Entity
    pub(crate) fn sync_inputs_from_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.data.current().content.clone();
        let title = self.data.current().title.clone();
        self.input.update(cx, |state, cx| {
            state.set_value(content, window, cx);
        });
        self.title_input.update(cx, |state, cx| {
            state.set_value(title, window, cx);
        });
    }

    /// 延迟 400ms 防抖保存，避免逐字输入时高频克隆并写入 SQLite
    fn schedule_debounced_save(&mut self, cx: &mut Context<Self>) {
        self.save_debounce_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(400))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.save_debounce_task = None;
                StickyModel::save(&this.data, cx);
            });
        }));
    }

    /// 立即落盘并取消尚未执行的防抖任务
    pub(crate) fn save_now(&mut self, cx: &mut Context<Self>) {
        self.save_debounce_task = None;
        StickyModel::save(&self.data, cx);
    }

    /// 失焦时如仍有未落盘的防抖写入则立刻冲刷
    fn flush_pending_save(&mut self, cx: &mut Context<Self>) {
        if self.save_debounce_task.take().is_some() {
            StickyModel::save(&self.data, cx);
        }
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 主题颜色适配（仅在色值实际变化时写入全局 Theme，避免每帧触发全局观察者通知）
        let current_color_index = self.data.current().color_index.min(STICKY_THEMES.len() - 1);
        let is_dark_theme = STICKY_THEMES[current_color_index].is_dark;
        let (fg, muted_fg, caret, selection) = if is_dark_theme {
            (
                hsla(0.0, 0.0, 0.90, 1.0),
                hsla(0.0, 0.0, 0.65, 1.0),
                hsla(0.0, 0.0, 0.92, 1.0),
                hsla(0.0, 0.0, 1.0, 0.18),
            )
        } else {
            (
                hsla(0.0, 0.0, 0.12, 1.0),
                hsla(0.0, 0.0, 0.40, 1.0),
                hsla(0.0, 0.0, 0.10, 1.0),
                hsla(0.0, 0.0, 0.0, 0.15),
            )
        };
        let colors = &gpui_component::Theme::global(cx).colors;
        if colors.foreground != fg
            || colors.muted_foreground != muted_fg
            || colors.caret != caret
            || colors.selection != selection
        {
            let theme_mut = gpui_component::Theme::global_mut(cx);
            theme_mut.colors.foreground = fg;
            theme_mut.colors.muted_foreground = muted_fg;
            theme_mut.colors.caret = caret;
            theme_mut.colors.selection = selection;
        }

        let current_idx = self.data.current_index;

        div()
            .relative()
            .flex()
            .flex_row()
            .items_stretch()
            .size_full()
            .min_h_0()
            .overflow_hidden()
            .on_drop(cx.listener(|this, paths: &gpui::ExternalPaths, _, cx| {
                let mut added = false;
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
                        added = true;
                    }
                }
                if added {
                    this.save_now(cx);
                    cx.notify();
                }
            }))
            // ── 左侧：便签 Tab 侧边栏 ──
            .child(render_sidebar(
                &self.data.notes,
                current_idx,
                &self.tab_scroll_handle,
                |this, window, cx, idx| {
                    if this.data.current_index != idx {
                        this.data.switch_to(idx);
                        this.sync_inputs_from_current(window, cx);
                        this.save_now(cx);
                        cx.notify();
                    }
                },
                |this, window, cx| {
                    this.data.new_note();
                    this.sync_inputs_from_current(window, cx);
                    this.save_now(cx);
                    this.tab_scroll_handle.scroll_to_bottom();
                    cx.notify();
                },
                |this, window, cx, from_idx, to_idx| {
                    if this.data.reorder_note(from_idx, to_idx) {
                        this.sync_inputs_from_current(window, cx);
                        this.save_now(cx);
                        cx.notify();
                    }
                },
                cx,
            ))
            // ── 右侧：便签纸主内容区 ──
            .child(render_content_panel(self, cx))
    }
}
