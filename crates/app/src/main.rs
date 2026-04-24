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

    let (tray_icon, toggle_id, quit_id) = tray::setup_tray().expect("系统托盘初始化失败");

    use gpui_component_assets::Assets;
    let app = Application::new().with_assets(Assets);
    let store_for_app = Arc::clone(&store);

    app.run(move |cx| {
        gpui_component::init(cx);
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

        // 提取所有 HWND 并注册到 thread_local（三步走，不嵌套）
        let store_for_hwnd = Arc::clone(&store_for_app);
        cx.spawn(async move |cx| {
            cx.background_executor().timer(std::time::Duration::from_millis(300)).await;

            // Step 1: 取出所有 handle（释放 WindowManager borrow）
            let (plugin_handles, main_handle): (Vec<(String, AnyWindowHandle)>, Option<AnyWindowHandle>) =
                match cx.update_global::<WindowManager, _>(|wm, _| {
                    let ph = wm.widget_windows.iter()
                        .map(|(id, (h, _))| (id.to_string(), h.clone()))
                        .collect();
                    let mh = wm.main_window.as_ref().map(|h| h.clone().into());
                    (ph, mh)
                }) { Ok(v) => v, Err(_) => return };

            // Step 2: 逐个读 HWND（无 WindowManager borrow）
            let mut id_hwnd: Vec<(String, isize)> = Vec::new();
            for (id, h) in &plugin_handles {
                let hwnd = cx.update(|cx| {
                    h.update(cx, |_, win, _| {
                        use raw_window_handle::HasWindowHandle;
                        if let Ok(wh) = win.window_handle() {
                            if let raw_window_handle::RawWindowHandle::Win32(h) = wh.as_raw() {
                                return h.hwnd.get() as isize;
                            }
                        }
                        0isize
                    }).unwrap_or(0)
                }).unwrap_or(0);
                if hwnd != 0 {
                    println!("[main] 插件 {} HWND = {}", id, hwnd);
                    id_hwnd.push((id.clone(), hwnd));
                }
            }

            let main_hwnd = if let Some(mh) = main_handle {
                cx.update(|cx| {
                    mh.update(cx, |_, win, _| {
                        use raw_window_handle::HasWindowHandle;
                        if let Ok(wh) = win.window_handle() {
                            if let raw_window_handle::RawWindowHandle::Win32(h) = wh.as_raw() {
                                return h.hwnd.get() as isize;
                            }
                        }
                        0isize
                    }).unwrap_or(0)
                }).unwrap_or(0)
            } else { 0 };

            if main_hwnd != 0 { println!("[main] 主窗口 HWND = {}", main_hwnd); }

            // Step 3: 写回 WindowManager + 注册 thread_local
            let _ = cx.update_global::<WindowManager, _>(|wm, _| {
                for (id, hwnd) in &id_hwnd {
                    if let Some(e) = wm.widget_windows.get_mut(id.as_str()) { e.1 = *hwnd; }
                    // 注册到 thread_local，供 widget-ui on_click 直接使用
                    widget_core::register_plugin_hwnd(id, *hwnd);
                }
                if main_hwnd != 0 { wm.main_hwnd = main_hwnd; }
            });

            let _ = store_for_hwnd;
        }).detach();

        // 托盘菜单事件轮询（仅此一个异步循环，操作简单不嵌套）
        let store_for_tray = Arc::clone(&store_for_app);
        cx.spawn(async move |cx| {
            let _tray = tray_icon;
            loop {
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == toggle_id {
                        // toggle_main_window_win32 纯 Win32，不嵌套
                        let next_visible = match cx.update_global::<WindowManager, _>(|wm, _| {
                            wm.toggle_main_window_win32()
                        }) { Ok(v) => v, Err(_) => true };

                        let _ = cx.update_global::<widget_core::UIState, _>(|s, _| {
                            s.is_visible = next_visible;
                        });
                        let _ = cx.update(|cx| cx.refresh_windows());

                    } else if event.id == quit_id {
                        let store_quit = Arc::clone(&store_for_tray);
                        // save_all_plugin_bounds 只调 cx.try_global + cx.set_global（无嵌套）
                        let _ = cx.update_global::<WindowManager, _>(|wm, cx| {
                            wm.save_all_plugin_bounds(cx, &store_quit);
                        });
                        drop(_tray);
                        let _ = cx.update(|cx| cx.quit());
                        break;
                    }
                }
                cx.background_executor().timer(std::time::Duration::from_millis(50)).await;
            }
        }).detach();
    });
}
