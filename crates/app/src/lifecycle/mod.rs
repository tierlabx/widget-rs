use crate::config::store::Store;
use crate::plugin::manager::PluginManager;
use crate::window::manager::WindowManager;
use gpui::*;
use std::sync::Arc;
use tray_icon::menu::MenuEvent;

pub fn spawn_hwnd_polling_task(cx: &mut App, store: Arc<Store>) {
    let store_for_hwnd = store;
    cx.spawn(async move |cx| {
        let mut id_hwnd: Vec<(String, isize)> = Vec::new();
        let mut captured_main_hwnd = 0isize;

        for _ in 0..50 {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(100))
                .await;

            // 1. 尝试提取主窗口 HWND
            if captured_main_hwnd == 0 {
                captured_main_hwnd = cx
                    .update(|cx| {
                        let main_handle = cx
                            .try_global::<WindowManager>()
                            .and_then(|wm| wm.main_window);
                        if let Some(h) = main_handle {
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
                        } else {
                            0isize
                        }
                    })
                    .unwrap_or(0);

                if captured_main_hwnd != 0 {
                    let _ = cx.update_global::<WindowManager, _>(|wm, _| {
                        wm.main_hwnd = captured_main_hwnd;
                    });
                    println!("[main] 主窗口 HWND = {}", captured_main_hwnd);
                }
            }

            // 2. 尝试提取所有插件窗口 HWND
            let plugin_handles: Vec<(String, AnyWindowHandle)> = match cx
                .update_global::<WindowManager, _>(|wm, _| {
                    wm.widget_windows
                        .iter()
                        .map(|(id, (h, _, _))| (id.to_string(), *h))
                        .collect()
                }) {
                Ok(v) => v,
                Err(_) => return,
            };

            let mut all_ready = captured_main_hwnd != 0;
            id_hwnd.clear();

            for (id, h) in &plugin_handles {
                let hwnd = cx
                    .update(|cx| {
                        h.update(cx, |_, win, _| {
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
                    .unwrap_or(0);
                if hwnd == 0 {
                    all_ready = false;
                    break;
                } else {
                    id_hwnd.push((id.clone(), hwnd));
                }
            }

            if all_ready && (!plugin_handles.is_empty() || captured_main_hwnd != 0) {
                break;
            }
        }

        for (id, hwnd) in &id_hwnd {
            println!("[main] 插件 {} HWND = {}", id, hwnd);
        }

        let _ = cx.update_global::<WindowManager, _>(|wm, cx| {
            let config = cx.try_global::<widget_core::AppConfig>().cloned();
            for (id, hwnd) in &id_hwnd {
                if let Some(e) = wm.widget_windows.get_mut(id.as_str()) {
                    e.1 = *hwnd;
                }
                widget_core::register_plugin_hwnd(id, *hwnd);
                let owner_hwnd = crate::window::platform::windows::apply_plugin_window_styles(
                    *hwnd,
                    id.as_str(),
                    config.as_ref(),
                );
                if let Some(e) = wm.widget_windows.get_mut(id.as_str()) {
                    e.2 = owner_hwnd;
                }

                if let Some((px, py, pw, ph)) =
                    widget_core::get_saved_physical_bounds(cx, id.as_str())
                {
                    unsafe {
                        windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                            *hwnd,
                            0,
                            px,
                            py,
                            pw,
                            ph,
                            windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                                | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                        );
                    }
                    println!(
                        "[main] SetWindowPos 精确修正 {} -> ({}, {}) {}x{}",
                        id, px, py, pw, ph
                    );
                }

                if let Some(cfg) = &config {
                    if let Some(p_cfg) = cfg.plugins.get(id.as_str()) {
                        if !p_cfg.enabled {
                            crate::window::platform::windows::show_plugin_window(*hwnd, false);
                        }
                    }
                }
            }
        });

        let _ = store_for_hwnd;
    })
    .detach();
}

pub fn spawn_tray_polling_task(
    cx: &mut App,
    tray_handles: crate::tray::TrayHandles,
    store: Arc<Store>,
) {
    let store_for_tray = store;
    cx.spawn(async move |cx| {
        let _tray = tray_handles.tray_icon;
        let toggle_item = tray_handles.toggle_item;
        let toggle_id = tray_handles.toggle_id;
        let quit_id = tray_handles.quit_id;

        let mut last_click_time = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(10))
            .unwrap_or_else(std::time::Instant::now);
        let mut last_visible = true;

        loop {
            // 1. 处理右键菜单事件
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == toggle_id {
                    let next_visible = toggle_main_panel(&cx);
                    last_visible = next_visible;
                    toggle_item.set_text(if next_visible {
                        "隐藏控制面板"
                    } else {
                        "显示控制面板"
                    });
                } else if event.id == quit_id {
                    let store_quit = Arc::clone(&store_for_tray);
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
                    return;
                }
            }

            // 2. 处理托盘图标点击事件（支持左键单击/双击切换）
            while let Ok(tray_event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                match tray_event {
                    tray_icon::TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Left,
                        button_state: tray_icon::MouseButtonState::Up,
                        ..
                    } => {
                        let now = std::time::Instant::now();
                        if now.duration_since(last_click_time)
                            > std::time::Duration::from_millis(250)
                        {
                            last_click_time = now;
                            let next_visible = toggle_main_panel(&cx);
                            last_visible = next_visible;
                            toggle_item.set_text(if next_visible {
                                "隐藏控制面板"
                            } else {
                                "显示控制面板"
                            });
                        }
                    }
                    _ => {}
                }
            }

            // 3. 动态检测主窗口实际状态并同步菜单文字与全局状态（例如被用户点击关闭按钮隐藏时）
            let (main_hwnd, is_wm_visible) = cx
                .update_global::<WindowManager, _>(|wm, _| (wm.main_hwnd, wm.is_visible))
                .unwrap_or((0, true));

            if main_hwnd != 0 {
                let actual_visible = unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(main_hwnd) != 0
                        && windows_sys::Win32::UI::WindowsAndMessaging::IsIconic(main_hwnd) == 0
                };
                if actual_visible != last_visible || actual_visible != is_wm_visible {
                    last_visible = actual_visible;
                    let _ = cx.update_global::<widget_core::UIState, _>(|s, _| {
                        s.is_visible = actual_visible;
                    });
                    let _ = cx.update_global::<WindowManager, _>(|wm, _| {
                        wm.is_visible = actual_visible;
                    });
                    toggle_item.set_text(if actual_visible {
                        "隐藏控制面板"
                    } else {
                        "显示控制面板"
                    });
                }
            }

            cx.background_executor()
                .timer(std::time::Duration::from_millis(200))
                .await;
        }
    })
    .detach();
}

/// 辅助函数：切换主控制面板窗口显示/隐藏状态
fn toggle_main_panel(cx: &AsyncApp) -> bool {
    // 确保 main_hwnd 存在；若为 0 尝试即时从 main_window 句柄获取
    let _ = cx.update(|cx| {
        let (main_hwnd, main_handle) = cx
            .try_global::<WindowManager>()
            .map(|wm| (wm.main_hwnd, wm.main_window))
            .unwrap_or((0, None));

        if main_hwnd == 0 {
            if let Some(h) = main_handle {
                let extracted = h
                    .update(cx, |_, win, _| {
                        use raw_window_handle::HasWindowHandle;
                        if let Ok(wh) = win.window_handle() {
                            if let raw_window_handle::RawWindowHandle::Win32(hw) = wh.as_raw() {
                                return hw.hwnd.get();
                            }
                        }
                        0isize
                    })
                    .unwrap_or(0);
                if extracted != 0 {
                    cx.update_global::<WindowManager, _>(|wm, _| {
                        wm.main_hwnd = extracted;
                    });
                }
            }
        }
    });

    let next_visible = cx
        .update_global::<WindowManager, _>(|wm, _| wm.toggle_main_window_win32())
        .unwrap_or(true);

    let _ = cx.update_global::<widget_core::UIState, _>(|s, _| {
        s.is_visible = next_visible;
    });
    let _ = cx.update(|cx| cx.refresh_windows());

    next_visible
}
