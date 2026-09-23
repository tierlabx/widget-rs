use gpui::DisplayId;

/// 显示器物理矩形区域 (左, 上, 右, 下)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    #[inline]
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    #[inline]
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    #[inline]
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    /// 计算与另一个矩形的重叠相交面积
    pub fn overlap_area(&self, other: &Rect) -> i64 {
        let inter_left = self.left.max(other.left);
        let inter_top = self.top.max(other.top);
        let inter_right = self.right.min(other.right);
        let inter_bottom = self.bottom.min(other.bottom);

        let w = (inter_right - inter_left).max(0) as i64;
        let h = (inter_bottom - inter_top).max(0) as i64;
        w * h
    }
}

/// 完整显示器信息（物理像素坐标及 DPI）
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// 完整显示区域（含任务栏）
    pub rc_monitor: Rect,
    /// 可用工作区域（排除任务栏和固定停靠栏）
    pub rc_work: Rect,
    /// 水平 DPI
    pub dpi_x: u32,
    /// 垂直 DPI
    pub dpi_y: u32,
    /// 缩放系数 (DPI / 96.0)
    pub scale_factor: f32,
    /// 是否为主显示器
    pub is_primary: bool,
}

/// 解析后的小组件窗口边界与目标显示器（用于 GPUI 初始窗口创建）
#[derive(Debug, Clone)]
pub struct ResolvedPluginBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub display_id: Option<DisplayId>,
}
