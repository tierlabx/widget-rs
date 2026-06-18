#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plugin_manager;
mod store;
mod tray;
mod window_manager;

use gpui::*;
use plugin_manager::PluginManager;
use std::sync::Arc;
use store::Store;
use tray_icon::menu::MenuEvent;
use widget_core::AppConfig;
use window_manager::WindowManager;

#[derive(rust_embed::RustEmbed)]
#[folder = "../../assets"]
struct LocalAssets;

struct AppAssets;

impl gpui::AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if let Some(file) = LocalAssets::get(path) {
            return Ok(Some(file.data));
        }
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        let mut list = gpui_component_assets::Assets.list(path).unwrap_or_default();
        for file in LocalAssets::iter() {
            if file.starts_with(path) {
                list.push(file.to_string().into());
            }
        }
        Ok(list)
    }
}

fn main() {
    // 1. 初始化存储和加载配置
    let store = Arc::new(Store::new());
    let mut config = store.load_config();
    println!("[main] 已加载配置: {:?}", config);

    // 同步开机自启动状态（比如安装包勾选了自启动，或者用户手动在注册表删了）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_str) = exe_path.to_str() {
            let exe_path_quoted = format!("\"{}\"", exe_str);
            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
            if let Ok(run_key) = hkcu.open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                winreg::enums::KEY_ALL_ACCESS,
            ) {
                let current_val: Result<String, _> = run_key.get_value("WidgetRS");
                let mut is_enabled = false;
                
                let _ = run_key.delete_value("Widget RS");

                if let Ok(val) = current_val {
                    if val == exe_path_quoted {
                        is_enabled = true;
                    } else if val.contains(exe_str) {
                        let _ = run_key.set_value("WidgetRS", &exe_path_quoted);
                        is_enabled = true;
                    }
                }

                if config.auto_start != is_enabled {
                    config.auto_start = is_enabled;
                    store.save_config(&config);
                    println!(
                        "[main] 开机自启动状态与系统注册表不一致，已同步配置 auto_start = {}",
                        is_enabled
                    );
                }
            }
        }
    }

    // 2. 初始化插件管理器并注册内置小组件
    let mut pm = PluginManager::new();
    pm.register(Arc::new(sticky_plugin::StickyWidgetPlugin));
    pm.register(Arc::new(todo_plugin::TodoWidgetPlugin));

    // 3. 初始化系统托盘（包括托盘图标和菜单）
    let (tray_icon, toggle_id, quit_id) = tray::setup_tray().expect("系统托盘初始化失败");

    let app = Application::new().with_assets(AppAssets);
    let store_for_app = Arc::clone(&store);

    app.run(move |cx| {
        // 初始化全局状态和组件资产
        gpui_component::init(cx);
        cx.set_global(config.clone());

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
                let _ = cx.update_global::<WindowManager, _>(|wm, cx| {
                    wm.save_all_plugin_bounds(cx, &store_for_bounds);
                });
            },
        )));

        // 初始化窗口管理器，用于管理主窗口和所有插件窗口的生命周期和状态
        WindowManager::init(cx);

        // 启动并注册所有已加载的插件窗口
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
            cx.background_executor()
                .timer(std::time::Duration::from_millis(300))
                .await;

            // Step 1: 取出所有 handle（释放 WindowManager borrow）
            let (plugin_handles, main_handle): (
                Vec<(String, AnyWindowHandle)>,
                Option<AnyWindowHandle>,
            ) = match cx.update_global::<WindowManager, _>(|wm, _| {
                let ph = wm
                    .widget_windows
                    .iter()
                    .map(|(id, (h, _))| (id.to_string(), h.clone()))
                    .collect();
                let mh = wm.main_window.as_ref().map(|h| h.clone().into());
                (ph, mh)
            }) {
                Ok(v) => v,
                Err(_) => return,
            };

            // Step 2: 逐个读 HWND（无 WindowManager borrow）
            let mut id_hwnd: Vec<(String, isize)> = Vec::new();
            for (id, h) in &plugin_handles {
                let hwnd = cx
                    .update(|cx| {
                        h.update(cx, |_, win, _| {
                            use raw_window_handle::HasWindowHandle;
                            if let Ok(wh) = win.window_handle() {
                                if let raw_window_handle::RawWindowHandle::Win32(h) = wh.as_raw() {
                                    return h.hwnd.get() as isize;
                                }
                            }
                            0isize
                        })
                        .unwrap_or(0)
                    })
                    .unwrap_or(0);
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
                    })
                    .unwrap_or(0)
                })
                .unwrap_or(0)
            } else {
                0
            };

            if main_hwnd != 0 {
                println!("[main] 主窗口 HWND = {}", main_hwnd);
            }

            // Step 3: 将 HWND 写回 WindowManager 并注册到 thread_local 供全局访问
            let _ = cx.update_global::<WindowManager, _>(|wm, cx| {
                let config = cx.try_global::<widget_core::AppConfig>().cloned();
                for (id, hwnd) in &id_hwnd {
                    if let Some(e) = wm.widget_windows.get_mut(id.as_str()) {
                        e.1 = *hwnd;
                    }
                    // 注册到 thread_local，供 widget-ui on_click 等跨线程操作直接使用
                    widget_core::register_plugin_hwnd(id, *hwnd);
                    // 防止 Win + D （显示桌面）操作导致小组件被隐藏
                    WindowManager::attach_to_desktop(*hwnd);
                    
                    // 恢复独立设置（置顶和鼠标穿透）
                    if let Some(cfg) = &config {
                        if let Some(plugin_cfg) = cfg.plugins.get(id.as_str()) {
                            unsafe {
                                use windows_sys::Win32::UI::WindowsAndMessaging::{
                                    SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
                                    GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_TRANSPARENT,
                                };
                                // 恢复始终置顶
                                let insert_after = if plugin_cfg.always_on_top { HWND_TOPMOST } else { HWND_NOTOPMOST };
                                SetWindowPos(*hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);

                                // 恢复鼠标穿透
                                let style = GetWindowLongW(*hwnd, GWL_EXSTYLE);
                                SetWindowLongW(
                                    *hwnd,
                                    GWL_EXSTYLE,
                                    if plugin_cfg.mouse_passthrough {
                                        style | WS_EX_TRANSPARENT as i32
                                    } else {
                                        style & !(WS_EX_TRANSPARENT as i32)
                                    },
                                );
                            }
                        }
                    }
                }
                if main_hwnd != 0 {
                    wm.main_hwnd = main_hwnd;
                }
            });

            let _ = store_for_hwnd;
        })
        .detach();

        // 启动托盘菜单事件的独立轮询循环（这是一个简单的轮询异步任务，避免借用嵌套）
        let store_for_tray = Arc::clone(&store_for_app);
        cx.spawn(async move |cx| {
            let _tray = tray_icon;
            loop {
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == toggle_id {
                        // toggle_main_window_win32 纯 Win32，不嵌套
                        let next_visible = match cx.update_global::<WindowManager, _>(|wm, _| {
                            wm.toggle_main_window_win32()
                        }) {
                            Ok(v) => v,
                            Err(_) => true,
                        };

                        let _ = cx.update_global::<widget_core::UIState, _>(|s, _| {
                            s.is_visible = next_visible;
                        });
                        let _ = cx.update(|cx| cx.refresh_windows());
                    } else if event.id == quit_id {
                        let store_quit = Arc::clone(&store_for_tray);
                        // 退出前，保存所有插件窗口的当前位置和状态。
                        // 这里直接操作 try_global / set_global，不涉及复杂的锁嵌套
                        let _ = cx.update_global::<WindowManager, _>(|wm, cx| {
                            wm.save_all_plugin_bounds(cx, &store_quit);
                        });
                        drop(_tray);
                        let _ = cx.update(|cx| cx.quit());
                        break;
                    }
                }
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(50))
                    .await;
            }
        })
        .detach();
    });
}
