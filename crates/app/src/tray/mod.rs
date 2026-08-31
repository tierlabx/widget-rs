use tray_icon::{
    menu::{Menu, MenuId, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

/// 从生成的 icon.png 加载托盘图标
///
/// 将图片文件解码为 RGBA 格式，并创建 `Icon` 实例用于系统托盘。
fn build_icon() -> Icon {
    let icon_bytes = include_bytes!("../../../../assets/logos/icon.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon image")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon")
}

pub struct TrayHandles {
    pub tray_icon: TrayIcon,
    pub toggle_item: MenuItem,
    pub toggle_id: MenuId,
    pub quit_id: MenuId,
}

/// 配置并初始化系统托盘
///
/// 创建包含“隐藏/显示控制面板”和“退出”功能的右键菜单，并构建系统托盘图标。
///
/// # 返回值
/// 成功时返回包含托盘句柄与菜单项引用的 `TrayHandles` 结构体。
pub fn setup_tray() -> Result<TrayHandles, Box<dyn std::error::Error>> {
    let tray_menu = Menu::new();

    // 默认应用启动时主控制面板是可见的，因此初始文案为“隐藏控制面板”
    let toggle_i = MenuItem::new("隐藏控制面板", true, None);
    let quit_i = MenuItem::new("退出 Widget RS", true, None);

    let toggle_id = toggle_i.id().clone();
    let quit_id = quit_i.id().clone();

    tray_menu.append_items(&[&toggle_i, &PredefinedMenuItem::separator(), &quit_i])?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Widget RS")
        .with_icon(build_icon())
        .build()?;

    Ok(TrayHandles {
        tray_icon,
        toggle_item: toggle_i,
        toggle_id,
        quit_id,
    })
}
