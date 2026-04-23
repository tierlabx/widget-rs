mod ui;
mod store;
mod window_manager;
mod tray;
mod plugin_manager;

use store::Store;
use window_manager::WindowManager;
use plugin_manager::PluginManager;
use tray_icon::menu::MenuEvent;
use gpui::*;

fn main() {
    let store = Store::new();
    let config = store.load_config();
    println!("Loaded config: {:?}", config);

    let mut pm = PluginManager::new();
    pm.register(std::sync::Arc::new(sticky_plugin::StickyWidgetPlugin));
    pm.register(std::sync::Arc::new(todo_plugin::TodoWidgetPlugin));

    // 初始化系统托盘
    let (tray_icon, toggle_id, quit_id) = tray::setup_tray().expect("Failed to init tray");

    // 初始化 GPUI 应用
    let app = Application::new();
    app.run(move |cx| {
        gpui_component::init(cx);
        WindowManager::init(cx);

        // 启动插件窗口
        let plugins = pm.get_plugins().to_vec();
        cx.update_global::<WindowManager, _>(|wm, cx| {
            for plugin in plugins {
                let handle = plugin.spawn_window(cx);
                wm.register_widget_window(plugin.id(), handle);
            }
        });

        // 托盘菜单事件异步轮询
        cx.spawn(async move |cx| {
            // 将 tray_icon 移动到这里以保持它的生命周期
            let _tray = tray_icon;
            loop {
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == toggle_id {
                        let _: gpui::Result<()> = cx.update_global::<WindowManager, _>(|wm, cx| {
                            wm.toggle_main_window(cx);
                        });
                    } else if event.id == quit_id {
                        // 显式丢弃托盘图标，防止出现重影
                        drop(_tray);
                        let _: gpui::Result<()> = cx.update(|cx| {
                            cx.quit();
                        });
                        break;
                    }
                }
                cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
            }
        }).detach();
    });
}

