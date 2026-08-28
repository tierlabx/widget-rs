use gpui::*;

use crate::{start_window_drag, update_window_edit_mode, UIState};

/// 插件内容 trait — 插件只需实现此 trait，由 WidgetWindow 统一包装
///
/// 实现该 trait 后，插件无需自行处理编辑模式检测、拖拽条渲染、
/// 窗口边框切换等窗口级通用逻辑，这些全部由 `WidgetWindow<T>` 统一管理。
pub trait WidgetContent: Render + Sized + 'static {
    /// 返回此内容对应的插件 ID（与 Plugin::id 一致）
    fn plugin_id(&self) -> &'static str;

    /// 返回编辑模式拖拽条上的标签文字（如 "拖拽移动便签"、"拖拽移动待办"）
    fn drag_label(&self) -> &'static str {
        "拖拽移动"
    }

    /// 是否显示编辑模式拖拽条（默认跟随编辑模式）
    ///
    /// 某些插件在特定状态下不希望显示拖拽条（如 stretchly 休息中），
    /// 可覆盖此方法返回 `false`。
    fn show_drag_handle(&self) -> bool {
        true
    }
}

/// 小组件窗口通用容器
///
/// 封装所有小组件窗口共享的行为：
/// - 编辑模式检测与窗口样式切换（`WS_THICKFRAME`）
/// - 编辑模式拖拽条（绿色 `#00d992`，支持原生窗口拖拽）
/// - 编辑模式边框高亮
///
/// 插件只需提供一个实现了 `WidgetContent` 的内容组件，
/// `WidgetWindow` 会自动在其上方叠加窗口级 UI。
pub struct WidgetWindow<T: WidgetContent> {
    content: Entity<T>,
}

impl<T: WidgetContent> WidgetWindow<T> {
    /// 创建一个新的小组件窗口容器
    ///
    /// # 参数
    /// * `content` - 实现了 `WidgetContent` 的插件内容 Entity
    pub fn new(content: Entity<T>) -> Self {
        Self { content }
    }
}

impl<T: WidgetContent> Render for WidgetWindow<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit_mode = cx.try_global::<UIState>().is_some_and(|s| s.is_edit_mode);

        // 统一更新窗口的编辑模式样式（WS_THICKFRAME 切换）
        update_window_edit_mode(window, is_edit_mode);

        let content = self.content.read(cx);
        let plugin_id = content.plugin_id();
        let drag_label = content.drag_label();
        let show_drag = content.show_drag_handle();

        // 编辑模式拖拽条
        let drag_handle = if is_edit_mode && show_drag {
            Some(
                div()
                    .w_full()
                    .h(px(28.0))
                    .bg(rgb(0x00d992)) // Emerald Signal Green
                    .flex()
                    .justify_center()
                    .items_center()
                    .flex_shrink_0()
                    .id(SharedString::from(format!("{}-drag", plugin_id)))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgba(0x00d992cc)))
                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                        start_window_drag(window);
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x050507))
                            .child(format!(":: {} ::", drag_label)),
                    ),
            )
        } else {
            None
        };

        let root = div().flex().flex_col().size_full();
        let root = if is_edit_mode {
            root.border_1().border_color(rgb(0x00d992))
        } else {
            root
        };

        root.children(drag_handle).child(self.content.clone())
    }
}

/// 创建标准小组件窗口选项（纯透明背景，适合有自身背景色的组件如 Sticky）
///
/// 封装所有插件窗口共用的 `WindowOptions` 样板配置：
/// 无标题栏、透明背景、PopUp 类型、不可缩放。
/// 位置由 `resolve_plugin_bounds` 从已保存配置中恢复。
///
/// # 参数
/// * `cx` - GPUI 应用上下文
/// * `plugin_id` - 插件 ID，用于从配置中读取已保存位置
/// * `default_bounds` - 默认位置和大小 `(x, y, width, height)`
pub fn default_widget_window_options(
    cx: &App,
    plugin_id: &str,
    default_bounds: (f32, f32, f32, f32),
) -> WindowOptions {
    let (x, y, w, h) = crate::resolve_plugin_bounds(cx, plugin_id, default_bounds);

    WindowOptions {
        titlebar: None,
        window_background: WindowBackgroundAppearance::Transparent,
        kind: WindowKind::PopUp,
        is_resizable: false,
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            Point::new(px(x), px(y)),
            size(px(w), px(h)),
        ))),
        ..Default::default()
    }
}

/// 创建标准小组件窗口选项（Acrylic 毛玻璃模糊背景，适合 Todo/Fences 等半透明组件）
///
/// 使用 `WindowBackgroundAppearance::Blurred` 启用 Windows Acrylic 效果，
/// 插件 view 只需在根 div 上设置半透明背景色叠加即可实现磨砂玻璃效果。
///
/// # 参数
/// * `cx` - GPUI 应用上下文
/// * `plugin_id` - 插件 ID，用于从配置中读取已保存位置
/// * `default_bounds` - 默认位置和大小 `(x, y, width, height)`
pub fn default_widget_window_options_blurred(
    cx: &App,
    plugin_id: &str,
    default_bounds: (f32, f32, f32, f32),
) -> WindowOptions {
    let (x, y, w, h) = crate::resolve_plugin_bounds(cx, plugin_id, default_bounds);

    WindowOptions {
        titlebar: None,
        window_background: WindowBackgroundAppearance::Blurred,
        kind: WindowKind::PopUp,
        is_resizable: false,
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            Point::new(px(x), px(y)),
            size(px(w), px(h)),
        ))),
        ..Default::default()
    }
}
