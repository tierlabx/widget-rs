use gpui::*;

pub struct UIState {
    pub is_visible: bool,
}

impl Global for UIState {}

pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle;
}
