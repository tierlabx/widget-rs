mod store;
mod window_manager;
mod tray;
mod plugin_manager;

use store::Store;
use window_manager::WindowManager;
use plugin_manager::PluginManager;
use tray_icon::menu::MenuEvent;
use gpui::*;
use std::sync::Arc;

fn main() {
    let store = Arc::new(Store::new());
    let config = store.load_config();
    println!("[main] 已加载配置: {:?}", config);

    let mut pm = PluginManager::new();
    pm.register(Arc::new(sticky_plugin::StickyWidgetPlugin));
    pm.register(Arc::new(todo_plugin::TodoWidgetPlugin));

    // 初始化系统托盘
    let (tray_icon, toggle_id, quit_id) = tray::setup_tray().expect("系统托盘初始化失败");

    // 初始化 GPUI 应用
    use gpui_component_assets::Assets;
    let app = Application::new().with_assets(Assets);
    let store_for_app = Arc::clone(&store);

    app.run(move |cx| {
        gpui_component::init(cx);

        // 将配置注入全局状态（插件通过 cx.try_global::<AppConfig>() 读取位置）
        cx.set_global(config.clone());

        WindowManager::init(cx);

        // 启动插件窗口
        let plugins = pm.get_plugins().to_vec();
        cx.update_global::<WindowManager, _>(|wm, cx| {
            for plugin in &plugins {
                let handle = plugin.spawn_window(cx);
                wm.register_widget_window(plugin.id(), handle);
            }
        });

        // 在插件窗口创建后，通过 window.update 提取每个插件的 HWND
        // 因为窗口渲染是异步的，我们在下一帧通过定时任务读取 HWND
        let store_for_hwnd = Arc::clone(&store_for_app);
        cx.spawn(async move |cx| {
            // 等待一帧，确保插件窗口已完成初始渲染
            cx.background_executor().timer(std::time::Duration::from_millis(200)).await;

            // 从 WindowManager 中遍历所有已注册的插件窗口，获取其 HWND
            let _ = cx.update_global::<WindowManager, _>(|wm, _cx| {
                // 遍历所有插件窗口，通过窗口句柄获取 HWND
                for (id, (handle, hwnd_slot)) in wm.widget_windows.iter_mut() {
                    // 尝试通过 window.update 获取 HWND
                    let hwnd_result = handle.update(_cx, |_, window, _| {
                        use raw_window_handle::HasWindowHandle;
                        if let Ok(wh) = window.window_handle() {
                            if let raw_window_handle::RawWindowHandle::Win32(h) = wh.as_raw() {
                                return h.hwnd.get() as isize;
                            }
                        }
                        0isize
                    });
                    if let Ok(hwnd) = hwnd_result {
                        if hwnd != 0 {
                            *hwnd_slot = hwnd;
                            println!("[main] 插件 {} HWND = {}", id, hwnd);
                        }
                    }
                }
            });

            let _ = store_for_hwnd; // 延长生命周期
        }).detach();

        // 托盘菜单事件异步轮询
        let store_for_tray = Arc::clone(&store_for_app);
        cx.spawn(async move |cx| {
            let _tray = tray_icon;
            loop {
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == toggle_id {
                        let _: gpui::Result<()> = cx.update_global::<WindowManager, _>(|wm, cx| {
                            wm.toggle_main_window(cx);
                        });
                    } else if event.id == quit_id {
                        // 退出前保存所有插件位置
                        let store_quit = Arc::clone(&store_for_tray);
                        let _: gpui::Result<()> = cx.update_global::<WindowManager, _>(|wm, cx| {
                            wm.save_all_plugin_bounds(cx, &store_quit);
                        });

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
