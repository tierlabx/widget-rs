use gpui::*;

pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle;
}
