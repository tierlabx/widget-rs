mod widget_window;
pub use widget_window::{default_widget_window_options, WidgetContent, WidgetWindow};

use gpui::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

/// 原生级别的全局编辑模式状态，供 WindowProc 直接读取
pub static NATIVE_EDIT_MODE: AtomicBool = AtomicBool::new(false);

fn default_true() -> bool {
    true
}

fn default_scale() -> f32 {
    1.0
}

/// 单个插件的位置与大小配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginConfig {
    /// GPUI 逻辑坐标（仅用于近似创建窗口）
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// 保存时所在显示器的 DPI 缩放比例（用于跨 DPI 显示器恢复时纠偏）
    #[serde(default = "default_scale")]
    pub scale: f32,
    /// 物理像素坐标（用于 SetWindowPos 精确恢复位置）
    #[serde(default)]
    pub phys_x: i32,
    #[serde(default)]
    pub phys_y: i32,
    #[serde(default)]
    pub phys_w: i32,
    #[serde(default)]
    pub phys_h: i32,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub mouse_passthrough: bool,
    #[serde(default = "default_true")]
    pub loaded: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// 应用全局配置（可被序列化存储）
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub auto_start: bool,
    /// 各插件位置，键为插件 ID，例如 "sticky_widget"
    pub plugins: HashMap<String, PluginConfig>,
    /// 插件自定义数据
    #[serde(default)]
    pub plugin_data: HashMap<String, serde_json::Value>,
}

impl AppConfig {
    pub fn get_plugin_data<T: serde::de::DeserializeOwned>(&self, plugin_id: &str) -> Option<T> {
        self.plugin_data
            .get(plugin_id)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set_plugin_data<T: serde::Serialize>(&mut self, plugin_id: &str, data: &T) {
        if let Ok(value) = serde_json::to_value(data) {
            self.plugin_data.insert(plugin_id.to_string(), value);
        }
    }
}

impl Global for AppConfig {}

/// 保存回调：由 app crate 注册，插件调用以立即落盘
pub struct SaveCallback(pub std::sync::Arc<dyn Fn(&AppConfig) + Send + Sync>);
impl Global for SaveCallback {}

/// 插件位置保存回调
pub struct SaveBoundsCallback(pub std::sync::Arc<dyn Fn(&mut App)>);
impl Global for SaveBoundsCallback {}

/// 插件加载卸载切换回调
#[derive(Clone)]
pub struct TogglePluginCallback(pub std::sync::Arc<dyn Fn(&mut App, &str, bool)>);
impl Global for TogglePluginCallback {}

/// 打开插件设置窗口的回调
#[derive(Clone)]
pub struct OpenPluginSettingsCallback(pub std::sync::Arc<dyn Fn(&mut App, &str)>);
impl Global for OpenPluginSettingsCallback {}

/// 立即落盘：克隆数据后交给后台执行器执行 IO，不阻塞 GPUI 主线程
/// 在任何 GPUI 事件处理器（subscribe/listener）内都可安全调用
pub fn save_config_now(cx: &mut App) {
    // 1. 先克隆好所需数据（短暂借用，立即释放）
    let config = match cx.try_global::<AppConfig>() {
        Some(c) => c.clone(),
        None => return,
    };
    let save_fn = match cx.try_global::<SaveCallback>() {
        Some(cb) => cb.0.clone(), // 克隆 Arc，立即释放对 SaveCallback 的借用
        None => return,
    };

    // 2. 将 IO 操作派发到后台线程，完全绕开 GPUI RefCell
    cx.background_executor()
        .spawn(async move {
            save_fn(&config);
        })
        .detach();
}

pub fn save_bounds_now(cx: &mut App) {
    if let Some(cb) = cx.try_global::<SaveBoundsCallback>() {
        let cb = cb.0.clone();
        cb(cx);
    }
}

/// UI 运行时状态（不持久化）
pub struct UIState {
    pub is_visible: bool,
    pub is_edit_mode: bool,
    /// 每个插件的加载状态，键为插件 ID
    pub plugin_loaded: HashMap<String, bool>,
    /// 每个插件的启用状态，键为插件 ID
    pub plugin_enabled: HashMap<String, bool>,
}

impl UIState {
    /// 获取插件的加载状态（默认为 true）
    pub fn is_plugin_loaded(&self, plugin_id: &str) -> bool {
        *self.plugin_loaded.get(plugin_id).unwrap_or(&true)
    }

    /// 获取插件的启用状态（默认为 true）
    pub fn is_plugin_enabled(&self, plugin_id: &str) -> bool {
        *self.plugin_enabled.get(plugin_id).unwrap_or(&true)
    }
}

impl Global for UIState {}

/// 插件 trait
pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;

    fn name(&self) -> &'static str {
        self.id()
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn icon(&self) -> gpui_component::IconName {
        gpui_component::IconName::WindowMaximize
    }

    fn version(&self) -> &'static str {
        "v1.0.0"
    }

    fn author(&self) -> &'static str {
        "官方 (内置)"
    }

    fn estimated_memory(&self) -> usize {
        0
    }

    #[allow(unused_variables)]
    fn on_load(&self, cx: &mut App) {}

    fn spawn_window(&self, cx: &mut App) -> AnyWindowHandle;

    #[allow(unused_variables)]
    fn on_unload(&self, cx: &mut App) {}

    #[allow(unused_variables)]
    fn build_settings_window(&self, cx: &mut App) {}
}

#[derive(Clone)]
pub struct PluginMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: gpui_component::IconName,
    pub version: &'static str,
    pub author: &'static str,
    pub estimated_memory: usize,
}

pub struct PluginList(pub Vec<PluginMetadata>);

impl Global for PluginList {}

// ─── 线程本地 HWND 存储 ───────────────────────────────────────────────────────
// 所有操作均在主线程执行，无需跨线程同步
// widget-ui 的 on_click 可直接读取 HWND 并调用 Win32 API，完全绕开 GPUI RefCell

thread_local! {
    static PLUGIN_HWNDS: RefCell<HashMap<String, isize>> = RefCell::new(HashMap::new());
    static ALL_PLUGIN_HWND_LIST: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

/// 注册插件 HWND（由 app crate 在 HWND 提取后调用）
pub fn register_plugin_hwnd(id: &str, hwnd: isize) {
    PLUGIN_HWNDS.with(|m| {
        m.borrow_mut().insert(id.to_string(), hwnd);
    });
    ALL_PLUGIN_HWND_LIST.with(|v| {
        let mut list = v.borrow_mut();
        if !list.contains(&hwnd) {
            list.push(hwnd);
        }
    });
}

/// 注销插件 HWND（由 app crate 在插件卸载时调用）
pub fn unregister_plugin_hwnd(id: &str) {
    let old_hwnd = PLUGIN_HWNDS.with(|m| m.borrow_mut().remove(id));
    if let Some(hwnd) = old_hwnd {
        ALL_PLUGIN_HWND_LIST.with(|v| {
            v.borrow_mut().retain(|h| *h != hwnd);
        });
    }
}

/// 获取指定插件的 HWND
pub fn get_plugin_hwnd(id: &str) -> isize {
    PLUGIN_HWNDS.with(|m| *m.borrow().get(id).unwrap_or(&0))
}

/// 获取所有插件的 HWND 列表（用于批量操作）
pub fn get_all_plugin_hwnds() -> Vec<isize> {
    ALL_PLUGIN_HWND_LIST.with(|v| v.borrow().clone())
}

pub fn start_window_drag(window: &mut Window) {
    use raw_window_handle::HasWindowHandle;
    if let Ok(handle) = HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
            unsafe {
                windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                    h.hwnd.get(),
                    windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN,
                    windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize,
                    0,
                );
            }
        }
    }
}

pub fn update_window_edit_mode(window: &mut Window, is_edit_mode: bool) {
    use raw_window_handle::HasWindowHandle;
    if let Ok(handle) = HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
            let hwnd = h.hwnd.get();
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_STYLE, SWP_FRAMECHANGED,
                    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_THICKFRAME,
                };
                let style = GetWindowLongW(hwnd, GWL_STYLE);
                if is_edit_mode {
                    if (style & WS_THICKFRAME as i32) == 0 {
                        SetWindowLongW(hwnd, GWL_STYLE, style | WS_THICKFRAME as i32);
                        SetWindowPos(
                            hwnd,
                            0,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                        );
                    }
                } else if (style & WS_THICKFRAME as i32) != 0 {
                    SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_THICKFRAME as i32));
                    SetWindowPos(
                        hwnd,
                        0,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                    );
                }
            }
        }
    }
}

/// 恢复插件窗口的近似逻辑坐标（用于 GPUI 初始窗口创建）
///
/// 此函数只做显示器边界校验，返回的逻辑坐标仅用于让 GPUI 在正确的显示器上创建窗口。
/// 精确的物理位置修正由 `get_saved_physical_bounds` + `SetWindowPos` 完成。
pub fn resolve_plugin_bounds(
    cx: &App,
    plugin_id: &str,
    default: (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    let plugin_cfg = cx
        .try_global::<AppConfig>()
        .and_then(|cfg| cfg.plugins.get(plugin_id).cloned());

    let Some(p) = plugin_cfg else {
        return default;
    };

    // 如果保存的尺寸为 0，说明是新创建的插件，直接用默认值
    if p.width <= 0.0 || p.height <= 0.0 {
        return default;
    }

    // 优先使用物理坐标进行显示器校验（更精确）
    let (check_cx, check_cy) = if p.phys_w > 0 && p.phys_h > 0 {
        (
            p.phys_x as f32 + p.phys_w as f32 / 2.0,
            p.phys_y as f32 + p.phys_h as f32 / 2.0,
        )
    } else {
        // 旧版 config 没有 phys_* 字段，用逻辑坐标 * scale 推算
        let s = if p.scale > 0.0 { p.scale } else { 1.0 };
        (p.x * s + p.width * s / 2.0, p.y * s + p.height * s / 2.0)
    };

    // 枚举所有活跃显示器，检查窗口中心点是否在某个显示器上
    let monitors = enumerate_monitors();
    for (i, m) in monitors.iter().enumerate() {
        println!(
            "[resolve_plugin_bounds] 显示器{}: ({},{})~({},{}) DPI={} 缩放={}%",
            i,
            m.left,
            m.top,
            m.right,
            m.bottom,
            m.dpi,
            (m.dpi as f32 / 96.0 * 100.0) as u32
        );
    }
    let on_monitor = monitors.iter().any(|m| {
        check_cx >= m.left as f32
            && check_cx < m.right as f32
            && check_cy >= m.top as f32
            && check_cy < m.bottom as f32
    });

    if on_monitor {
        // 位置有效，返回保存的逻辑坐标（GPUI 近似定位）
        // 精确修正由后续的 SetWindowPos 完成
        (p.x, p.y, p.width, p.height)
    } else {
        println!(
            "[resolve_plugin_bounds] 插件 {} 物理中心 ({}, {}) 不在任何活跃显示器上，回退到默认位置",
            plugin_id, check_cx, check_cy
        );
        default
    }
}

/// 获取插件已保存的物理像素坐标（经过显示器边界校验）
///
/// 返回 `Some((x, y, w, h))` 表示保存的物理坐标仍在有效显示器上，可用于 `SetWindowPos` 精确恢复。
/// 返回 `None` 表示坐标无效（新插件、显示器已断开等），不应调用 `SetWindowPos`。
pub fn get_saved_physical_bounds(cx: &App, plugin_id: &str) -> Option<(i32, i32, i32, i32)> {
    let p = cx
        .try_global::<AppConfig>()
        .and_then(|cfg| cfg.plugins.get(plugin_id).cloned())?;

    // 必须有有效的物理坐标
    if p.phys_w <= 0 || p.phys_h <= 0 {
        return None;
    }

    let cx_pt = p.phys_x as f32 + p.phys_w as f32 / 2.0;
    let cy_pt = p.phys_y as f32 + p.phys_h as f32 / 2.0;

    let monitors = enumerate_monitors();
    let on_monitor = monitors.iter().any(|m| {
        cx_pt >= m.left as f32
            && cx_pt < m.right as f32
            && cy_pt >= m.top as f32
            && cy_pt < m.bottom as f32
    });

    if on_monitor {
        Some((p.phys_x, p.phys_y, p.phys_w, p.phys_h))
    } else {
        None
    }
}

/// 显示器信息（物理像素坐标）
struct MonitorRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    dpi: u32,
}

/// 枚举所有活跃显示器，返回物理像素坐标和 DPI
fn enumerate_monitors() -> Vec<MonitorRect> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::{BOOL, RECT};
        use windows_sys::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MONITORINFO,
        };

        struct State {
            monitors: Vec<MonitorRect>,
        }

        unsafe extern "system" fn callback(
            hmon: isize,
            _hdc: isize,
            _lp_rect: *mut RECT,
            lparam: isize,
        ) -> BOOL {
            let s = &mut *(lparam as *mut State);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(hmon, &mut info as *mut _);
            let r = info.rcWork; // 使用工作区域（排除任务栏）

            // 获取该显示器的 DPI
            let mut dpi_x: u32 = 96;
            let mut dpi_y: u32 = 96;
            let _ = windows_sys::Win32::UI::HiDpi::GetDpiForMonitor(
                hmon, 0, // MDT_EFFECTIVE_DPI
                &mut dpi_x, &mut dpi_y,
            );

            s.monitors.push(MonitorRect {
                left: r.left,
                top: r.top,
                right: r.right,
                bottom: r.bottom,
                dpi: dpi_x,
            });
            1
        }

        let mut state = State {
            monitors: Vec::new(),
        };

        unsafe {
            EnumDisplayMonitors(
                0,
                std::ptr::null(),
                Some(callback),
                &mut state as *mut State as isize,
            );
        }

        state.monitors
    }

    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}
