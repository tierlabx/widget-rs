use std::time::Instant;

use crate::item_card::resolve_file_visual;
use crate::model::{FenceCategory, FenceItem, FencesData};

#[test]
fn test_fences_default_categories() {
    let data = FencesData::default();
    assert_eq!(data.categories.len(), 3);
    assert_eq!(data.categories[0].name, "程序");
    assert_eq!(data.categories[1].name, "文件夹");
    assert_eq!(data.categories[2].name, "文件");
}

#[test]
fn test_resolve_file_visual() {
    // 1. 文件夹
    let dir_vis = resolve_file_visual("C:\\Projects", true);
    assert!(matches!(
        dir_vis.icon_name,
        gpui_component::IconName::Folder
    ));
    assert!(dir_vis.badge_text.is_none());

    // 2. 可执行程序 / 快捷方式
    let exe_vis = resolve_file_visual("C:\\Program Files\\App.exe", false);
    assert_eq!(exe_vis.badge_text.as_deref(), Some("EXE"));

    let lnk_vis = resolve_file_visual("C:\\Users\\Desktop\\App.lnk", false);
    assert_eq!(lnk_vis.badge_text.as_deref(), Some("LNK"));

    // 3. 代码文件
    let rs_vis = resolve_file_visual("src/main.rs", false);
    assert_eq!(rs_vis.badge_text.as_deref(), Some("RS"));

    // 4. 文档类型
    let pdf_vis = resolve_file_visual("document.pdf", false);
    assert_eq!(pdf_vis.badge_text.as_deref(), Some("PDF"));

    let xls_vis = resolve_file_visual("table.xlsx", false);
    assert_eq!(xls_vis.badge_text.as_deref(), Some("XLS"));
}

#[test]
fn test_category_item_reordering() {
    let mut cat = FenceCategory {
        name: "测试栏".to_string(),
        items: vec![
            FenceItem {
                name: "Item 0".to_string(),
                path: "C:\\0.txt".to_string(),
                is_dir: false,
            },
            FenceItem {
                name: "Item 1".to_string(),
                path: "C:\\1.txt".to_string(),
                is_dir: false,
            },
            FenceItem {
                name: "Item 2".to_string(),
                path: "C:\\2.txt".to_string(),
                is_dir: false,
            },
        ],
        collapsed: false,
    };

    // 模拟从 index 0 移动到 index 2
    let moved = cat.items.remove(0);
    cat.items.insert(2, moved);

    assert_eq!(cat.items[0].name, "Item 1");
    assert_eq!(cat.items[1].name, "Item 2");
    assert_eq!(cat.items[2].name, "Item 0");
}

#[test]
fn bench_fences_visual_resolution() {
    let test_paths = [
        ("C:\\Apps\\editor.exe", false),
        ("C:\\Docs\\notes.md", false),
        ("C:\\Workspace\\code.rs", false),
        ("C:\\Pictures\\photo.png", false),
        ("C:\\Music\\song.mp3", false),
        ("C:\\Movies\\film.mp4", false),
        ("C:\\Archives\\data.zip", false),
        ("C:\\Windows\\System32", true),
    ];

    let iterations = 20_000;
    let start = Instant::now();
    for i in 0..iterations {
        let (path, is_dir) = test_paths[i % test_paths.len()];
        let _ = resolve_file_visual(path, is_dir);
    }
    let duration = start.elapsed();
    println!(
        "[性能测试] 解析 {iterations} 次文件视觉样式耗时: {:?}",
        duration
    );
    assert!(
        duration.as_millis() < 100,
        "20000次文件类型解析应在100ms内完成"
    );
}
