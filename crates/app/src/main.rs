#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod assets;
mod config;
mod lifecycle;
mod plugin;
mod system;
mod tray;
mod window;

use config::Store;
use gpui::*;
use plugin::manager::PluginManager;
use std::sync::Arc;
use widget_core::AppConfig;
use window::manager::WindowManager;
fn main() {
    // 0. 初始化崩溃日志捕获机制（确保启动阶段及后续运行期的任何 panic/崩溃均能记录到本地文件）
    system::crash_handler::init_crash_handler();

    // 1. 初始化存储和加载配置
    let store = Arc::new(Store::new());
    let mut app_config = store.load_config();
    // println!("[main] 已加载配置: {:?}", app_config);

    system::autostart::sync_auto_start_with_registry(&mut app_config, &store);

    // 2. 初始化插件管理器并注册内置小组件
    let mut pm = PluginManager::new();
    plugin::registry::register_all_plugins(&mut pm);

    // 3. 初始化系统托盘（包括托盘图标和菜单）
    let tray_handles = tray::setup_tray(app_config.silent_start).expect("系统托盘初始化失败");

    let app = Application::new().with_assets(assets::AppAssets);
    let store_for_app = Arc::clone(&store);

    app.run(move |cx| {
        // 初始化全局状态和组件资产
        gpui_component::init(cx);
        // 全局启用深色主题（让所有 Input 输入框、文字、光标默认呈现纯白色）
        gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);
        // 将 gpui_component Root 默认背景色改为 100% 全透明，杜绝小组件窗口被刷上浅色主题纯白底板
        gpui_component::Theme::global_mut(cx).colors.background = gpui::hsla(0.0, 0.0, 0.0, 0.0);

        cx.set_global(app_config.clone());

        // 提取并存储全局 PluginList 元数据
        let metadata_list = pm
            .get_plugins()
            .iter()
            .map(|p| widget_core::PluginMetadata {
                id: p.id(),
                name: p.name(),
                description: p.description(),
                icon: p.icon(),
                version: p.version(),
                author: p.author(),
                estimated_memory: p.estimated_memory(),
                has_settings: p.has_settings(),
            })
            .collect::<Vec<_>>();
        cx.set_global(widget_core::PluginList(metadata_list));

        // 注册更新状态桥接（异步任务通过此全局变量回传更新检查/下载状态）
        cx.set_global(widget_ui::MainWindowUpdateBridge {
            status: widget_ui::UpdateStatus::Idle,
            dismissed: false,
        });

        // 注册立即写盘回调，插件可调用 save_config_now(cx) 触发
        let store_for_save = Arc::clone(&store_for_app);
        cx.set_global(widget_core::SaveCallback(std::sync::Arc::new(
            move |cfg: &AppConfig| {
                store_for_save.save_config(cfg);
            },
        )));

        let store_for_bounds = Arc::clone(&store_for_app);
        cx.set_global(widget_core::SaveBoundsCallback(std::sync::Arc::new(
            move |cx: &mut App| {
                cx.update_global::<WindowManager, _>(|wm, cx| {
                    wm.save_all_plugin_bounds(cx, &store_for_bounds);
                });
            },
        )));

        cx.set_global(widget_core::TogglePluginCallback(std::sync::Arc::new(
            move |cx: &mut App, plugin_id: &str, loaded: bool| {
                if let Some(pm) = cx.try_global::<PluginManager>().cloned() {
                    if let Some(plugin) = pm.get_plugins().iter().find(|p| p.id() == plugin_id) {
                        println!("[TogglePluginCallback] plugin_id: {}, loaded: {}", plugin_id, loaded);
                        if loaded {
                            let handle = plugin.spawn_window(cx);
                            println!("[TogglePluginCallback] spawn_window called!");
                            cx.update_global::<WindowManager, _>(|wm, _| {
                                wm.register_widget_window(plugin.id(), handle);
                            });

                            let plugin_id_string = plugin_id.to_string();
                            cx.spawn(async move |cx| {
                                let mut hwnd = 0;
                                for _ in 0..50 {
                                    cx.background_executor().timer(std::time::Duration::from_millis(100)).await;
                                    hwnd = cx.update(|cx| {
                                        let h = cx.try_global::<WindowManager>()
                                            .and_then(|wm| wm.widget_windows.get(&plugin_id_string.as_str()))
                                            .map(|(h, _, _)| *h);
                                        if let Some(h) = h {
                                            h.update(cx, |_, win, _| {
                                                use raw_window_handle::HasWindowHandle;
                                                if let Ok(wh) = win.window_handle() {
                                                    if let raw_window_handle::RawWindowHandle::Win32(hw) = wh.as_raw() {
                                                        return hw.hwnd.get();
                                                    }
                                                }
                                                0isize
                                            }).unwrap_or(0)
                                        } else { 0 }
                                    }).unwrap_or(0);
                                    if hwnd != 0 { break; }
                                }

                                if hwnd != 0 {
                                    let _ = cx.update(|cx| {
                                        cx.update_global::<WindowManager, _>(|wm, _| {
                                            if let Some(e) = wm.widget_windows.get_mut(plugin_id_string.as_str()) {
                                                e.1 = hwnd;
                                            }
                                        });
                                        widget_core::register_plugin_hwnd(&plugin_id_string, hwnd);
                                        let config = cx.try_global::<widget_core::AppConfig>().cloned();
                                        let owner_hwnd = window::platform::windows::apply_plugin_window_styles(hwnd, &plugin_id_string, config.as_ref());
                                        // 将 Owner HWND 存入 widget_windows
                                        cx.update_global::<WindowManager, _>(|wm, _| {
                                            if let Some(e) = wm.widget_windows.get_mut(plugin_id_string.as_str()) {
                                                e.2 = owner_hwnd;
                                            }
                                        });
                                        // 用保存的物理坐标精确修正窗口位置
                                        if let Some((px, py, pw, ph)) = widget_core::get_saved_physical_bounds(cx, &plugin_id_string) {
                                            unsafe {
                                                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                                                    hwnd,
                                                    0,
                                                    px, py, pw, ph,
                                                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                                                    | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                                                );
                                            }
                                            println!("[TogglePluginCallback] SetWindowPos 精确修正 {} -> ({}, {}) {}x{}", plugin_id_string, px, py, pw, ph);
                                        }
                                    });
                                }
                            }).detach();
                        } else {
                            // 卸载插件：先保存位置，再清理资源
                            // 通过 SaveBoundsCallback 保存所有插件位置（包含 store 引用）
                            let save_cb = cx
                                .try_global::<widget_core::SaveBoundsCallback>()
                                .map(|cb| cb.0.clone());
                            if let Some(cb) = save_cb {
                                cb(cx);
                            }
                            let (handle_opt, hwnd) = cx.update_global::<WindowManager, _>(|wm, _| {
                                let entry = wm.widget_windows.get(plugin_id);
                                let h = entry.map(|(handle, _, _)| *handle);
                                let hwnd = entry.map(|(_, hwnd, _)| *hwnd).unwrap_or(0);
                                wm.remove_widget_window(plugin_id); // 同时销毁 Owner
                                (h, hwnd)
                            });
                            // 清理 WndProc 子类化
                            window::platform::windows::cleanup_plugin_window_styles(hwnd);
                            // 注销 HWND
                            widget_core::unregister_plugin_hwnd(plugin_id);
                            // 销毁窗口
                            if let Some(handle) = handle_opt {
                                let _ = handle.update(cx, |_, window, _| {
                                    window.remove_window();
                                });
                            }
                            plugin.on_unload(cx);
                            cx.defer(|_| {
                                widget_core::trim_process_memory();
                            });
                        }
                    }
                }
            }
        )));

        cx.set_global(widget_core::OpenPluginSettingsCallback(std::sync::Arc::new(
            move |cx: &mut App, plugin_id: &str| {
                if let Some(pm) = cx.try_global::<PluginManager>().cloned() {
                    if let Some(plugin) = pm.get_plugins().iter().find(|p| p.id() == plugin_id) {
                        plugin.build_settings_window(cx);
                    }
                }
            }
        )));

        // 初始化窗口管理器，用于管理主窗口和所有插件窗口的生命周期和状态
        WindowManager::init(cx);

        // 启动并注册所有已加载的插件窗口
        let plugins = pm.get_plugins().to_vec();
        for plugin in &plugins {
            plugin.on_load(cx);
        }

        cx.update_global::<WindowManager, _>(|wm, cx| {
            let config = cx.try_global::<widget_core::AppConfig>().cloned();
            for plugin in &plugins {
                let is_loaded = config
                    .as_ref()
                    .and_then(|c| c.plugins.get(plugin.id()))
                    .map(|p| p.loaded)
                    .unwrap_or(true);
                if is_loaded {
                    let handle = plugin.spawn_window(cx);
                    wm.register_widget_window(plugin.id(), handle);
                }
            }
        });
        cx.set_global(pm);

        // 提取所有 HWND 并注册到 thread_local
        let store_for_hwnd = Arc::clone(&store_for_app);
        lifecycle::spawn_hwnd_polling_task(cx, store_for_hwnd);

        // 启动托盘菜单事件的独立轮询循环
        let store_for_tray = Arc::clone(&store_for_app);
        lifecycle::spawn_tray_polling_task(cx, tray_handles, store_for_tray);

        // 若开启了自动检查更新，则在后台异步发起新版本检查
        if app_config.auto_check_update {
            widget_ui::check_for_update(cx);
        }
    });
}
