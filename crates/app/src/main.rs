#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod plugin_manager;
mod plugin_registry;
mod store;
mod tray;
mod window_manager;

use gpui::*;
use plugin_manager::PluginManager;
use std::sync::{Arc, Mutex, OnceLock};
use store::Store;
use tray_icon::menu::MenuEvent;
use widget_core::AppConfig;
use window_manager::WindowManager;

static WND_PROCS: OnceLock<Mutex<std::collections::HashMap<isize, isize>>> = OnceLock::new();

unsafe extern "system" fn plugin_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let old_proc = {
        let procs = WND_PROCS.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        let guard = procs.lock().unwrap();
        *guard.get(&hwnd).unwrap_or(&0)
    };

    if old_proc == 0 {
        return windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(
            hwnd, msg, wparam, lparam,
        );
    }

    let old_proc_fn: unsafe extern "system" fn(isize, u32, usize, isize) -> isize =
        std::mem::transmute(old_proc);

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_WINDOWPOSCHANGING
        && widget_core::NATIVE_EDIT_MODE.load(std::sync::atomic::Ordering::SeqCst)
    {
        unsafe {
            use windows_sys::Win32::Graphics::Gdi::{
                GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::WINDOWPOS;
            let pos = &mut *(lparam as *mut WINDOWPOS);
            if (pos.flags & windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOMOVE) == 0 {
                let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                let mut info: MONITORINFO = std::mem::zeroed();
                info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(hmonitor, &mut info) != 0 {
                    let snap = 20;
                    let work_rect = info.rcWork;

                    if (pos.x - work_rect.left).abs() < snap {
                        pos.x = work_rect.left;
                    } else if (work_rect.right - (pos.x + pos.cx)).abs() < snap {
                        pos.x = work_rect.right - pos.cx;
                    }

                    if (pos.y - work_rect.top).abs() < snap {
                        pos.y = work_rect.top;
                    } else if (work_rect.bottom - (pos.y + pos.cy)).abs() < snap {
                        pos.y = work_rect.bottom - pos.cy;
                    }
                }
            }
        }
    }

    let res = windows_sys::Win32::UI::WindowsAndMessaging::CallWindowProcW(
        Some(old_proc_fn),
        hwnd,
        msg,
        wparam,
        lparam,
    );

    if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_NCHITTEST
        && !widget_core::NATIVE_EDIT_MODE.load(std::sync::atomic::Ordering::SeqCst)
    {
        if let 10..=17 = res {
            return 1; // HTCLIENT
        }
    }

    res
}

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
    plugin_registry::register_all_plugins(&mut pm);

    // 3. 初始化系统托盘（包括托盘图标和菜单）
    let (tray_icon, toggle_id, quit_id) = tray::setup_tray().expect("系统托盘初始化失败");

    let app = Application::new().with_assets(AppAssets);
    let store_for_app = Arc::clone(&store);

    app.run(move |cx| {
        // 初始化全局状态和组件资产
        gpui_component::init(cx);
        cx.set_global(config.clone());

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
            })
            .collect::<Vec<_>>();
        cx.set_global(widget_core::PluginList(metadata_list));

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

        // 初始化窗口管理器，用于管理主窗口和所有插件窗口的生命周期和状态
        WindowManager::init(cx);

        // 启动并注册所有已加载的插件窗口
        let plugins = pm.get_plugins().to_vec();
        for plugin in &plugins {
            plugin.on_load(cx);
        }

        cx.update_global::<WindowManager, _>(|wm, cx| {
            for plugin in &plugins {
                let handle = plugin.spawn_window(cx);
                wm.register_widget_window(plugin.id(), handle);
            }
        });
        cx.set_global(pm);

        // 提取所有 HWND 并注册到 thread_local（三步走，不嵌套）
        let store_for_hwnd = Arc::clone(&store_for_app);
        cx.spawn(async move |cx| {
            let mut id_hwnd: Vec<(String, isize)> = Vec::new();
            let mut main_hwnd = 0;

            // 使用重试循环等待所有窗口句柄准备完毕
            for _ in 0..50 {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(100))
                    .await;

                // Step 1: 取出所有 handle（释放 WindowManager borrow）
                let (plugin_handles, main_handle): (
                    Vec<(String, AnyWindowHandle)>,
                    Option<AnyWindowHandle>,
                ) = match cx.update_global::<WindowManager, _>(|wm, _| {
                    let ph = wm
                        .widget_windows
                        .iter()
                        .map(|(id, (h, _))| (id.to_string(), *h))
                        .collect();
                    let mh = wm.main_window.as_ref().map(|h| (*h).into());
                    (ph, mh)
                }) {
                    Ok(v) => v,
                    Err(_) => return,
                };

                // Step 2: 逐个读 HWND（无 WindowManager borrow）
                let mut all_ready = true;
                id_hwnd.clear();

                for (id, h) in &plugin_handles {
                    let hwnd = cx
                        .update(|cx| {
                            h.update(cx, |_, win, _| {
                                use raw_window_handle::HasWindowHandle;
                                if let Ok(wh) = win.window_handle() {
                                    if let raw_window_handle::RawWindowHandle::Win32(h) =
                                        wh.as_raw()
                                    {
                                        return h.hwnd.get();
                                    }
                                }
                                0isize
                            })
                            .unwrap_or(0)
                        })
                        .unwrap_or(0);
                    if hwnd == 0 {
                        all_ready = false;
                        break;
                    } else {
                        id_hwnd.push((id.clone(), hwnd));
                    }
                }

                if !all_ready {
                    continue;
                }

                main_hwnd = if let Some(mh) = main_handle {
                    cx.update(|cx| {
                        mh.update(cx, |_, win, _| {
                            use raw_window_handle::HasWindowHandle;
                            if let Ok(wh) = win.window_handle() {
                                if let raw_window_handle::RawWindowHandle::Win32(h) = wh.as_raw() {
                                    return h.hwnd.get();
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
                    break; // 所有句柄都已成功获取
                }
            }

            for (id, hwnd) in &id_hwnd {
                println!("[main] 插件 {} HWND = {}", id, hwnd);
            }
            if main_hwnd != 0 {
                println!("[main] 主窗口 HWND = {}", main_hwnd);
            } else {
                println!("[main] 警告：未能获取主窗口 HWND");
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
                    // 移除默认的 WS_THICKFRAME 并子类化窗口以彻底禁用原生缩放
                    unsafe {
                        use windows_sys::Win32::UI::WindowsAndMessaging::{
                            GetWindowLongW, SetWindowLongPtrW, SetWindowLongW, SetWindowPos,
                            GWLP_WNDPROC, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
                            SWP_NOZORDER, WS_THICKFRAME,
                        };
                        let style = GetWindowLongW(*hwnd, GWL_STYLE);
                        SetWindowLongW(*hwnd, GWL_STYLE, style & !(WS_THICKFRAME as i32));
                        SetWindowPos(
                            *hwnd,
                            0,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                        );

                        // 注入自定义窗口过程
                        let old_proc = SetWindowLongPtrW(
                            *hwnd,
                            GWLP_WNDPROC,
                            plugin_wnd_proc as *const () as isize,
                        );
                        if old_proc != 0 {
                            let procs = WND_PROCS
                                .get_or_init(|| Mutex::new(std::collections::HashMap::new()));
                            procs.lock().unwrap().insert(*hwnd, old_proc);
                        }
                    }

                    // 恢复独立设置（置顶和鼠标穿透）
                    if let Some(cfg) = &config {
                        if let Some(plugin_cfg) = cfg.plugins.get(id.as_str()) {
                            unsafe {
                                use windows_sys::Win32::UI::WindowsAndMessaging::{
                                    SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE,
                                    SWP_NOSIZE,
                                };
                                // 恢复始终置顶
                                let insert_after = if plugin_cfg.always_on_top {
                                    HWND_TOPMOST
                                } else {
                                    HWND_NOTOPMOST
                                };
                                SetWindowPos(
                                    *hwnd,
                                    insert_after,
                                    0,
                                    0,
                                    0,
                                    0,
                                    SWP_NOMOVE | SWP_NOSIZE,
                                );

                                use windows_sys::Win32::UI::WindowsAndMessaging::{
                                    GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_LAYERED,
                                    WS_EX_TRANSPARENT,
                                };
                                let style = GetWindowLongW(*hwnd, GWL_EXSTYLE);
                                SetWindowLongW(
                                    *hwnd,
                                    GWL_EXSTYLE,
                                    if plugin_cfg.mouse_passthrough {
                                        style | WS_EX_TRANSPARENT as i32 | WS_EX_LAYERED as i32
                                    } else {
                                        style & !(WS_EX_TRANSPARENT as i32 | WS_EX_LAYERED as i32)
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
                // 托盘菜单事件
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == toggle_id {
                        // toggle_main_window_win32 纯 Win32，不嵌套
                        let next_visible = cx
                            .update_global::<WindowManager, _>(|wm, _| {
                                wm.toggle_main_window_win32()
                            })
                            .unwrap_or(true);

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
                        let _ = cx.update(|cx| {
                            if let Some(pm) = cx.try_global::<PluginManager>() {
                                let plugins = pm.get_plugins().to_vec();
                                for plugin in plugins {
                                    plugin.on_unload(cx);
                                }
                            }
                        });
                        drop(_tray);
                        let _ = cx.update(|cx| cx.quit());
                        break;
                    }
                }

                // 托盘图标左键点击事件
                if let Ok(tray_icon::TrayIconEvent::Click {
                    button,
                    button_state,
                    ..
                }) = tray_icon::TrayIconEvent::receiver().try_recv()
                {
                    if button == tray_icon::MouseButton::Left
                        && button_state == tray_icon::MouseButtonState::Up
                    {
                        let next_visible = cx
                            .update_global::<WindowManager, _>(|wm, _| {
                                wm.toggle_main_window_win32()
                            })
                            .unwrap_or(true);

                        let _ = cx.update_global::<widget_core::UIState, _>(|s, _| {
                            s.is_visible = next_visible;
                        });
                        let _ = cx.update(|cx| cx.refresh_windows());
                    }
                }

                cx.background_executor()
                    .timer(std::time::Duration::from_millis(200))
                    .await;
            }
        })
        .detach();
    });
}
