mod model;
pub mod overlay;
mod tips;
mod view;

use gpui::*;
use widget_core::Plugin;

use view::StretchlyWidget;

mod settings_view;
pub use settings_view::StretchlyLiveStats;
use settings_view::StretchlySettingsView;

/// 设置页通知组件「立即应用配置并重置计时器」的全局信号
pub struct StretchlyApplyNow(pub bool);
impl Global for StretchlyApplyNow {}

/// 全局信号：休息是否正在进行（供副屏遮罩窗口监听自毁）
pub struct StretchlyBreakActive(pub bool);
impl Global for StretchlyBreakActive {}

/// 每次 timer tick 更新的休息状态快照，供 BreakOverlay 渲染用
/// 避免 BreakOverlay 在 render() 里跨 Entity 借用 StretchlyWidget
#[derive(Clone, Default)]
pub struct StretchlyBreakSnapshot {
    pub state: model::BreakState,
    pub time_str: String,
    pub progress: f32,
    pub break_label: &'static str,
    pub break_duration_label: String,
    pub is_mini: bool,
    pub skip_available: bool,
    pub skip_label: String,
    pub postpone_mins: u64,
    pub tip: String,
    pub allow_skip: bool,
    pub allow_postpone: bool,
}
impl Global for StretchlyBreakSnapshot {}

/// BreakOverlay 按钮回调：通知 StretchlyWidget 执行操作
#[derive(Clone, PartialEq, Eq)]
pub enum StretchlyOverlayAction {
    Skip,
    Postpone,
}
pub struct StretchlyOverlayRequest(pub Option<StretchlyOverlayAction>);
impl Global for StretchlyOverlayRequest {}

pub struct StretchlyWidgetPlugin;

impl Plugin for StretchlyWidgetPlugin {
    fn id(&self) -> &'static str {
        "stretchly_widget"
    }

    fn name(&self) -> &'static str {
        "休息提醒"
    }

    fn description(&self) -> &'static str {
        "定期提醒你休息，保护视力和身体健康。"
    }

    fn icon(&self) -> gpui_component::IconName {
        gpui_component::IconName::CircleCheck
    }

    fn estimated_memory(&self) -> usize {
        1024 * 1024 // 1MB
    }

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle {
        let options = widget_core::default_widget_window_options(
            cx,
            "stretchly_widget",
            // 默认：右上角紧凑小组件，280x100
            (1250.0, 100.0, 280.0, 100.0),
        );

        cx.open_window(options, |window, cx| {
            let content = cx.new(|cx| StretchlyWidget::new(window, cx));
            let widget_window = cx.new(|_cx| widget_core::WidgetWindow::new(content));
            cx.new(|cx| gpui_component::Root::new(widget_window, window, cx))
        })
        .unwrap()
        .into()
    }

    fn on_unload(&self, cx: &mut App) {
        // 清理 stretchly 注册的所有全局状态，释放关联的堆内存
        cx.set_global(StretchlyApplyNow(false));
        cx.set_global(StretchlyBreakActive(false));
        cx.set_global(StretchlyBreakSnapshot::default());
        cx.set_global(StretchlyOverlayRequest(None));
        cx.set_global(StretchlyLiveStats(Default::default()));
        println!("[stretchly] on_unload: 已清理所有全局状态");
    }

    fn build_settings_window(&self, cx: &mut App) {
        let options = WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Opaque,
            kind: WindowKind::PopUp,
            is_resizable: true,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                Point::new(px(200.0), px(200.0)),
                size(px(420.0), px(740.0)),
            ))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| StretchlySettingsView::new(window, cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        })
        .unwrap();
    }
}

pub fn create_plugin() -> std::sync::Arc<dyn Plugin> {
    std::sync::Arc::new(StretchlyWidgetPlugin)
}
