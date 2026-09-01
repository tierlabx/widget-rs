use gpui::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// 创建标准插件设置弹窗窗口选项（暗黑主题不透明弹窗）
///
/// # 参数
/// * `_cx` - GPUI 应用上下文
/// * `size_tuple` - 初始宽高 `(width, height)`，例如 `(420.0, 680.0)`
pub fn default_settings_window_options(_cx: &App, size_tuple: (f32, f32)) -> WindowOptions {
    WindowOptions {
        titlebar: None,
        window_background: WindowBackgroundAppearance::Opaque,
        kind: WindowKind::PopUp,
        is_resizable: true,
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            Point::new(px(250.0), px(180.0)),
            size(px(size_tuple.0), px(size_tuple.1)),
        ))),
        ..Default::default()
    }
}

/// 渲染标准设置窗口标题栏（含左侧原生窗口拖拽区域与右侧标准关闭按钮）
pub fn render_settings_titlebar(title: impl Into<SharedString>) -> impl IntoElement {
    let title_str = title.into();
    div()
        .flex()
        .justify_between()
        .items_center()
        .w_full()
        .h(px(46.0))
        .flex_shrink_0()
        .bg(rgb(0x161b22))
        .border_b_1()
        .border_color(rgb(0x30363d))
        .child(
            div()
                .flex()
                .flex_1()
                .items_center()
                .h_full()
                .pl(px(16.0))
                .id("settings-titlebar-drag")
                .on_mouse_down(MouseButton::Left, |_, win, _| {
                    if let Ok(h) = win.window_handle() {
                        if let RawWindowHandle::Win32(h) = h.as_raw() {
                            unsafe {
                                windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                                windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                    h.hwnd.get(),
                                    windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN,
                                    windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize,
                                    0,
                                );
                            }
                        }
                    }
                })
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xe6edf3))
                        .child(title_str),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(46.0))
                .h_full()
                .hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                .text_color(rgb(0x8b949e))
                .cursor_pointer()
                .id("settings-close-btn")
                .on_click(|_, win, cx| {
                    win.remove_window();
                    cx.defer(|_| {
                        crate::trim_process_memory();
                    });
                })
                .child(gpui_component::Icon::new(gpui_component::IconName::Close)),
        )
}

/// 渲染标准设置区块小标题
pub fn settings_section_header(title: impl Into<SharedString>) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0x8b949e))
        .pt(px(4.0))
        .pb(px(2.0))
        .child(title.into())
}

/// 标准设置卡片容器（深色 GitHub Dark 风格 #161b22，边框 #30363d）
pub fn settings_card() -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .rounded(px(8.0))
        .bg(rgb(0x161b22))
        .border_1()
        .border_color(rgb(0x30363d))
}

/// 封装整个设置弹窗的标准外壳布局
///
/// 包含标准背景、统一边框、标题栏拖拽、右上角关闭按钮与内嵌滚动容器
pub fn render_settings_shell(title: impl Into<SharedString>, content: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(rgb(0x0d1117))
        .border_1()
        .border_color(rgb(0x30363d))
        .child(render_settings_titlebar(title))
        .child(
            div()
                .id("settings-scroll")
                .flex_1()
                .overflow_y_scroll()
                .child(content),
        )
}
