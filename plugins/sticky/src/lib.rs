use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Icon, IconName};
use raw_window_handle::HasWindowHandle;
use widget_core::{AppConfig, Plugin};

pub struct StickyWidget {
    hwnd_reported: bool,
    input: Entity<InputState>,
}

impl StickyWidget {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 从全局配置读取已保存的便签内容
        let saved_content = cx
            .try_global::<AppConfig>()
            .and_then(|c| c.get_plugin_data::<String>("sticky_widget"))
            .unwrap_or_default();

        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .default_value(saved_content)
                .placeholder("在这里记录你的想法...")
        });

        // 内容变化时更新内存 + 立即写盘
        // 便签内容较小，每次变化直接落盘是可接受的
        cx.subscribe(
            &input,
            |_this: &mut Self, input: Entity<InputState>, event: &InputEvent, cx| {
                // Change 事件在每次文字变化时触发
                if let InputEvent::Change = event {
                    let text = input.read(cx).value().to_string();
                    cx.update_global::<AppConfig, _>(|config, _| {
                        config.set_plugin_data("sticky_widget", &text);
                    });
                    widget_core::save_config_now(cx);
                }
            },
        )
        .detach();

        Self {
            hwnd_reported: false,
            input,
        }
    }
}

impl Render for StickyWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit_mode = cx
            .try_global::<widget_core::UIState>()
            .is_some_and(|s| s.is_edit_mode);

        if let Ok(handle) = _window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
                let hwnd = h.hwnd.get();
                if !self.hwnd_reported {
                    self.hwnd_reported = true;
                    let _ = hwnd;
                }
            }
        }
        widget_core::update_window_edit_mode(_window, is_edit_mode);

        let drag_handle = if is_edit_mode {
            Some(
                div()
                    .w_full()
                    .h(px(28.0))
                    .bg(rgb(0x00d992)) // Emerald Signal Green
                    .flex()
                    .justify_center()
                    .items_center()
                    .id("sticky-drag")
                    .cursor_pointer()
                    .hover(|s| s.bg(rgba(0x00d992cc)))
                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                        widget_core::start_window_drag(window);
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x050507))
                            .child(":: 拖拽移动便签 ::"),
                    ),
            )
        } else {
            None
        };

        let input = &self.input;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgba(0x050507d9))
            .border_1()
            .border_color(if is_edit_mode {
                rgb(0x00d992)
            } else {
                rgb(0x3d3a39)
            })
            .rounded(px(8.0))
            .children(drag_handle)
            // 标题栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .w_full()
                    .px(px(14.0))
                    .py(px(10.0))
                    .bg(rgb(0xfef3c7)) // 保持暖黄
                    .border_b_1()
                    .border_color(rgba(0xf59e0b80)) // 加深下划线
                    .child(
                        div()
                            .text_color(rgb(0x92400e))
                            .child(Icon::new(IconName::File).size(px(14.0))),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x78350f))
                            .child("便签"),
                    ),
            )
            // 文本区域
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .w_full()
                    .bg(rgb(0xfef3c7))
                    .p(px(8.0))
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .h_full()
                            .bg(rgba(0xffffffa0)) // 让便签输入框底色更白一些以示输入区
                            .border_1()
                            .border_color(rgba(0xf59e0b20))
                            .rounded(px(4.0))
                            .p(px(8.0))
                            .text_color(rgb(0x3d2000))
                            .hover(|s| s.border_color(rgba(0xf59e0b60))) // hover时有略深边框
                            .child(Input::new(input).h_full().appearance(false).bordered(false)),
                    ),
            )
    }
}

pub struct StickyWidgetPlugin;

impl Plugin for StickyWidgetPlugin {
    fn id(&self) -> &'static str {
        "sticky_widget"
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let (x, y, w, h) = cx
            .try_global::<AppConfig>()
            .and_then(|cfg| cfg.plugins.get("sticky_widget").cloned())
            .map(|p| (p.x, p.y, p.width, p.height))
            .unwrap_or((1250.0, 50.0, 320.0, 360.0));

        let options = WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::PopUp,
            is_resizable: false,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::new(px(x), px(y)),
                size(px(w), px(h)),
            ))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| StickyWidget::new(window, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        })
        .unwrap()
        .into()
    }
}
