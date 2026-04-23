use gpui::*;
use crate::ui::main_window::MainWindow;
use std::collections::HashMap;

pub struct WindowManager {
    pub main_window: Option<WindowHandle<gpui_component::Root>>,
    pub widget_windows: HashMap<&'static str, AnyWindowHandle>,
    pub is_visible: bool,
}

impl Global for WindowManager {}

impl WindowManager {
    pub fn init(cx: &mut App) {
        cx.set_global(Self {
            main_window: None,
            widget_windows: HashMap::new(),
            is_visible: true,
        });
        
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: None,
                appears_transparent: true,
                traffic_light_position: None,
            }),
            window_background: WindowBackgroundAppearance::Transparent,
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1200.0), px(800.0)), cx))),
            ..Default::default()
        };

        let window = cx.open_window(options, |window, cx| {
            let view = cx.new(|_| MainWindow::new());
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        }).unwrap();
        
        cx.update_global::<Self, _>(|wm, _cx| {
            wm.main_window = Some(window);
        });
    }

    pub fn register_widget_window(&mut self, id: &'static str, handle: AnyWindowHandle) {
        self.widget_windows.insert(id, handle);
    }

    pub fn toggle_main_window(&mut self, cx: &mut App) {
        self.is_visible = !self.is_visible;
        println!("Toggle main window requested. is_visible = {}", self.is_visible);
        
        if let Some(window) = &self.main_window {
            window.update(cx, |_, _, cx| {
                cx.notify();
            }).ok();
        }
    }
}
