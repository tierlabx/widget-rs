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
        for _ in 0..50 {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(100))
                .await;

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

            let mut all_ready = true;
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

            if all_ready && !id_hwnd.is_empty() {
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
    tray_icon: tray_icon::TrayIcon,
    toggle_id: tray_icon::menu::MenuId,
    quit_id: tray_icon::menu::MenuId,
    store: Arc<Store>,
) {
    let store_for_tray = store;
    cx.spawn(async move |cx| {
        let _tray = tray_icon;
        loop {
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == toggle_id {
                    let next_visible = cx
                        .update_global::<WindowManager, _>(|wm, _| wm.toggle_main_window_win32())
                        .unwrap_or(true);

                    let _ = cx.update_global::<widget_core::UIState, _>(|s, _| {
                        s.is_visible = next_visible;
                    });
                    let _ = cx.update(|cx| cx.refresh_windows());
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
                    break;
                }
            }

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
                        .update_global::<WindowManager, _>(|wm, _| wm.toggle_main_window_win32())
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
}
