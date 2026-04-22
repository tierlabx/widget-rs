use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem, MenuId},
    TrayIconBuilder, TrayIcon, Icon,
};

/// 生成一个 32x32 的翡翠绿实心圆图标（RGBA 原始数据）
fn build_icon() -> Icon {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let r = (size as f32 / 2.0) - 1.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let idx = ((y * size + x) * 4) as usize;
            if (dx * dx + dy * dy).sqrt() <= r {
                rgba[idx]     = 0x00; // R
                rgba[idx + 1] = 0xd9; // G
                rgba[idx + 2] = 0x92; // B
                rgba[idx + 3] = 0xff; // A
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}

/// 返回 (TrayIcon, toggle_menu_id, quit_menu_id)
pub fn setup_tray() -> Result<(TrayIcon, MenuId, MenuId), Box<dyn std::error::Error>> {
    let tray_menu = Menu::new();

    let toggle_i = MenuItem::new("Toggle Control Center", true, None);
    let quit_i   = MenuItem::new("Quit Widget RS", true, None);

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
