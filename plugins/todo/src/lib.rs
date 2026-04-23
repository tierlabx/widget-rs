use gpui::*;
use widget_core::Plugin;
use raw_window_handle::HasWindowHandle;

pub struct TodoWidget;

impl TodoWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Render for TodoWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit_mode = cx.try_global::<widget_core::UIState>().map_or(false, |s| s.is_edit_mode);

        if let Ok(handle) = _window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
                unsafe {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowLongW, SetWindowLongW, GWL_STYLE, WS_THICKFRAME};
                    let hwnd = h.hwnd.get() as isize;
                    let style = GetWindowLongW(hwnd, GWL_STYLE);
                    if is_edit_mode {
                        if (style & WS_THICKFRAME as i32) == 0 {
                            SetWindowLongW(hwnd, GWL_STYLE, style | WS_THICKFRAME as i32);
                        }
                    } else {
                        if (style & WS_THICKFRAME as i32) != 0 {
                            SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_THICKFRAME as i32));
                        }
                    }
                }
            }
        }

        let drag_handle = if is_edit_mode {
            Some(
                div()
                    .w_full()
                    .h(px(28.0))
                    .bg(rgb(0x00d992))
                    .flex()
                    .justify_center()
                    .items_center()
                    .id("todo-drag")
                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                        if let Ok(handle) = window.window_handle() {
                            if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
                                unsafe {
                                    windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                                    windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW(
                                        h.hwnd.get() as isize, 
                                        windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN, 
                                        windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize, 
                                        0
                                    );
                                }
                            }
                        }
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x050507))
                            .child(":: 拖拽移动待办 ::")
                    )
            )
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgba(0x101010d9)) // Carbon Surface transparent
            .border_1()
            .border_color(if is_edit_mode { rgb(0x00d992) } else { rgb(0x3d3a39) }) // Highlight border in edit mode
            .rounded(px(8.0))
            .children(drag_handle)
            .child(
                // todoList
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .p(px(16.0))
                    .gap(px(8.0))
                    // Item 1 (completed = false)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .w_full()
                            .px(px(14.0))
                            .py(px(12.0))
                            .gap(px(12.0))
                            .bg(rgb(0x050507))
                            .rounded(px(6.0))
                            .child(
                                // check
                                div()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .rounded_full()
                                    .border_2()
                                    .border_color(rgb(0x3d3a39))
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xf2f2f2))
                                    .child("完成项目设计")
                            )
                    )
                    // Item 2 (completed = true)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .w_full()
                            .px(px(14.0))
                            .py(px(12.0))
                            .gap(px(12.0))
                            .bg(rgb(0x050507))
                            .rounded(px(6.0))
                            .child(
                                // check
                                div()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .rounded_full()
                                    .bg(rgb(0x00d992))
                                    .flex()
                                    .justify_center()
                                    .items_center()
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x8b949e))
                                    .child("编写文档")
                            )
                    )
                    // Item 3 (completed = false)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .w_full()
                            .px(px(14.0))
                            .py(px(12.0))
                            .gap(px(12.0))
                            .bg(rgb(0x050507))
                            .rounded(px(6.0))
                            .child(
                                // check
                                div()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .rounded_full()
                                    .border_2()
                                    .border_color(rgb(0x3d3a39))
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xf2f2f2))
                                    .child("代码审查")
                            )
                    )
                    // Spacer
                    .child(div().flex_1())
                    // Add Btn
                    .child(
                        div()
                            .flex()
                            .justify_center()
                            .items_center()
                            .w_full()
                            .p(px(12.0))
                            .gap(px(8.0))
                            .bg(rgb(0x050507))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(rgb(0x3d3a39))
                            .child(
                                div()
                                    .text_lg()
                                    .text_color(rgb(0xb8b3b0))
                                    .child("+")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xb8b3b0))
                                    .child("添加待办")
                            )
                    )
            )
    }
}

pub struct TodoWidgetPlugin;

impl Plugin for TodoWidgetPlugin {
    fn id(&self) -> &'static str {
        "todo_widget"
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::PopUp,
            is_resizable: false,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::new(px(1250.0), px(450.0)),
                size(px(360.0), px(400.0)),
            ))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|_| TodoWidget::new());
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        }).unwrap().into()
    }
}

