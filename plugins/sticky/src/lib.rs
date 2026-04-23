use gpui::*;
use widget_core::{AppConfig, Plugin};
use raw_window_handle::HasWindowHandle;

pub struct StickyWidget {
    hwnd_reported: bool,
}

impl StickyWidget {
    pub fn new() -> Self {
        Self { hwnd_reported: false }
    }
}

impl Render for StickyWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit_mode = cx.try_global::<widget_core::UIState>().map_or(false, |s| s.is_edit_mode);

        if let Ok(handle) = _window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
                let hwnd = h.hwnd.get() as isize;

                // 上报 HWND 到 WindowManager（仅需执行一次）
                if !self.hwnd_reported {
                    self.hwnd_reported = true;
                    cx.update_global::<widget_core::UIState, _>(|_, _| {}); // 借用 cx 触发下面调用
                    // 通知 App 级别的 WindowManager 记录 HWND
                    // 注：此处通过 cx.app_mut() 不可直接访问，改用 emit 方式通知
                    // 实际在 spawn_window 后由 main.rs 负责首次记录
                    let _ = hwnd; // 稍后由 main.rs 通过 set_hwnd 处理
                }

                unsafe {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowLongW, SetWindowLongW, GWL_STYLE, WS_THICKFRAME};
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
            .bg(rgba(0x050507d9))
            .border_1()
            .border_color(if is_edit_mode { rgb(0x00d992) } else { rgb(0x3d3a39) })
            .rounded(px(8.0))
            .children(drag_handle)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .p(px(16.0))
                    .bg(rgba(0xfef3c7f2))
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
        // 尝试从已保存的配置中读取位置，否则使用默认值
        let (x, y, w, h) = cx
            .try_global::<AppConfig>()
            .and_then(|cfg| cfg.plugins.get("sticky_widget").cloned())
            .map(|p| (p.x, p.y, p.width, p.height))
            .unwrap_or((1250.0, 50.0, 320.0, 360.0));

        println!("[StickyPlugin] 初始位置: ({}, {}) {}x{}", x, y, w, h);

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
            let view = cx.new(|_| StickyWidget::new());
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        }).unwrap().into()
    }
}
