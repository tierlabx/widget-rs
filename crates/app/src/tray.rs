use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem, MenuId},
    TrayIconBuilder, TrayIcon, Icon,
};

/// 从生成的 icon.png 加载托盘图标
///
/// 将图片文件解码为 RGBA 格式，并创建 `Icon` 实例用于系统托盘。
fn build_icon() -> Icon {
    let icon_bytes = include_bytes!("../../../assets/logos/icon.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon image")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon")
}

/// 配置并初始化系统托盘
///
/// 创建包含“显示/隐藏控制台”和“退出”功能的右键菜单，并构建系统托盘图标。
///
/// # 返回值
/// 成功时返回包含三个元素的元组：
/// - `TrayIcon`: 托盘图标实例（需要保持存活）
/// - `MenuId`: “显示/隐藏”菜单项的唯一标识符
/// - `MenuId`: “退出”菜单项的唯一标识符
pub fn setup_tray() -> Result<(TrayIcon, MenuId, MenuId), Box<dyn std::error::Error>> {
    let tray_menu = Menu::new();

    let toggle_i = MenuItem::new("显示/隐藏控制台", true, None);
    let quit_i   = MenuItem::new("退出 Widget RS", true, None);

    let toggle_id = toggle_i.id().clone();
    let quit_id   = quit_i.id().clone();

    tray_menu.append_items(&[
        &toggle_i,
        &PredefinedMenuItem::separator(),
        &quit_i,
    ])?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Widget RS")
        .with_icon(build_icon())
        .build()?;

    Ok((tray_icon, toggle_id, quit_id))
}
