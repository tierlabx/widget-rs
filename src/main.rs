mod ui;
mod store;
mod window_manager;
mod tray;
mod plugin_manager;

use store::Store;
use window_manager::WindowManager;
use plugin_manager::PluginManager;
use tray_icon::menu::MenuEvent;
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    std::env::set_var("SLINT_STYLE", "fluent-dark");

    let store = Store::new();
    let config = store.load_config();
    println!("Loaded config: {:?}", config);

    let pm = PluginManager::new();
    println!("Discovered plugins: {:?}", pm.discover_plugins());

    let (_tray_icon, toggle_id, quit_id) = tray::setup_tray().expect("Failed to init tray");

    let wm = WindowManager::new()?;

    // 拦截主窗口关闭 → 隐藏而非退出
    wm.main_window().window().on_close_requested(|| {
        slint::CloseRequestResponse::HideWindow
    });

    // 显示所有窗口
    wm.show_all();

    // 安排在事件循环内执行任务栏修复（Timer 中获取有效 HWND）
    wm.schedule_taskbar_fix();

    // 托盘菜单事件轮询
    let main_weak = wm.main_window().as_weak();
    let tray_timer = slint::Timer::default();
    tray_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(50),
        move || {
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == toggle_id {
                    if let Some(mw) = main_weak.upgrade() {
                        let visible = mw.window().is_visible();
                        if visible { let _ = mw.window().hide(); }
                        else       { let _ = mw.show(); }
                    }
                } else if event.id == quit_id {
                    slint::quit_event_loop().unwrap();
                }
            }
        },
    );

    slint::run_event_loop()
}
