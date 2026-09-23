//! 显示器几何与 DPI 缩放适配模块
//!
//! 负责多显示器枚举、物理/逻辑坐标映射、目标显示器匹配与防离屏回退。

mod resolver;
mod types;
mod win32;

pub use resolver::{
    clamp_to_work_area, find_best_monitor, find_primary_monitor, get_saved_physical_bounds,
    match_display_for_logical_bounds, resolve_plugin_bounds, resolve_plugin_window_bounds,
};
pub use types::{MonitorInfo, Rect, ResolvedPluginBounds};
pub use win32::enumerate_monitors;

#[cfg(test)]
#[path = "../monitor_tests.rs"]
mod monitor_tests;
