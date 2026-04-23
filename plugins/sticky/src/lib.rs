use gpui::*;
use widget_core::Plugin;
use raw_window_handle::HasWindowHandle;

pub struct StickyWidget;

impl StickyWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Render for StickyWidget {
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
                    .id("sticky-drag")
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
                            .child(":: 拖拽移动便签 ::")
                    )
            )
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgba(0x050507d9)) // Abyss Black transparent
            .border_1()
            .border_color(if is_edit_mode { rgb(0x00d992) } else { rgb(0x3d3a39) }) // Highlight border in edit mode
            .rounded(px(8.0))
            .children(drag_handle)
            .child(
                // stickyContent
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .p(px(16.0))
                    .bg(rgba(0xfef3c7f2)) // warm yellow
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x78350f))
                            .child("在这里记录你的想法...\n\n双击编辑内容")
                    )
            )
    }
}

pub struct StickyWidgetPlugin;

impl Plugin for StickyWidgetPlugin {
    fn id(&self) -> &'static str {
        "sticky_widget"
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::PopUp,
            is_resizable: false,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::new(px(1250.0), px(50.0)),
                size(px(320.0), px(360.0)),
            ))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|_| StickyWidget::new());
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        }).unwrap().into()
    }
}

