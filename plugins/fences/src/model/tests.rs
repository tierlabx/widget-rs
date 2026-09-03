use std::time::Instant;

use super::{FenceCategory, FenceItem, FencesData};
use crate::system::favicon::extract_domain;
use crate::ui::visual::resolve_file_visual;

#[test]
fn test_fences_default_categories() {
    let data = FencesData::default();
    assert_eq!(data.categories.len(), 3);
    assert_eq!(data.categories[0].name, "程序");
    assert_eq!(data.categories[1].name, "文件夹");
    assert_eq!(data.categories[2].name, "文件");
}

#[test]
fn test_fences_merge_extra_categories() {
    let mut data = FencesData {
        categories: vec![
            FenceCategory {
                name: "程序".to_string(),
                items: vec![FenceItem {
                    name: "App".to_string(),
                    path: "app.exe".to_string(),
                    is_dir: false,
                }],
                collapsed: false,
            },
            FenceCategory {
                name: "文件夹".to_string(),
                items: vec![],
                collapsed: false,
            },
            FenceCategory {
                name: "文件".to_string(),
                items: vec![],
                collapsed: false,
            },
            FenceCategory {
                name: "网页书签".to_string(),
                items: vec![FenceItem {
                    name: "GitHub".to_string(),
                    path: "https://github.com".to_string(),
                    is_dir: false,
                }],
                collapsed: false,
            },
        ],
    };

    let mut extra_items = Vec::new();
    for extra_cat in data.categories.drain(3..) {
        extra_items.extend(extra_cat.items);
    }
    if let Some(first_cat) = data.categories.get_mut(0) {
        first_cat.items.extend(extra_items);
    }

    assert_eq!(data.categories.len(), 3);
    assert_eq!(data.categories[0].items.len(), 2);
    assert_eq!(data.categories[0].items[1].name, "GitHub");
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

    // 2. 网页书签与 URL
    let web_vis = resolve_file_visual("https://github.com", false);
    assert!(matches!(web_vis.icon_name, gpui_component::IconName::Globe));
    assert_eq!(web_vis.badge_text.as_deref(), Some("WEB"));

    let url_file_vis = resolve_file_visual("C:\\Users\\Desktop\\Google.url", false);
    assert!(matches!(
        url_file_vis.icon_name,
        gpui_component::IconName::Globe
    ));
    assert_eq!(url_file_vis.badge_text.as_deref(), Some("URL"));

    let item = FenceItem {
        name: "GitHub".to_string(),
        path: "https://github.com".to_string(),
        is_dir: false,
    };
    assert!(item.is_web_url());

    // 3. 可执行程序 / 快捷方式
    let exe_vis = resolve_file_visual("C:\\Program Files\\App.exe", false);
    assert_eq!(exe_vis.badge_text.as_deref(), Some("EXE"));

    let lnk_vis = resolve_file_visual("C:\\Users\\Desktop\\App.lnk", false);
    assert_eq!(lnk_vis.badge_text.as_deref(), Some("LNK"));

    // 4. 代码文件
    let rs_vis = resolve_file_visual("src/main.rs", false);
    assert_eq!(rs_vis.badge_text.as_deref(), Some("RS"));

    // 5. 文档类型
    let pdf_vis = resolve_file_visual("document.pdf", false);
    assert_eq!(pdf_vis.badge_text.as_deref(), Some("PDF"));

    let xls_vis = resolve_file_visual("table.xlsx", false);
    assert_eq!(xls_vis.badge_text.as_deref(), Some("XLS"));
}

#[test]
fn test_extract_domain() {
    assert_eq!(
        extract_domain("https://github.com/rust-lang/rust"),
        Some("github.com".to_string())
    );
    assert_eq!(
        extract_domain("http://crates.io/crates/gpui"),
        Some("crates.io".to_string())
    );
    assert_eq!(
        extract_domain("https://bilibili.com:443/video?id=123#comment"),
        Some("bilibili.com:443".to_string())
    );
    assert_eq!(extract_domain("google.com"), Some("google.com".to_string()));
    assert_eq!(extract_domain(""), None);
}

#[test]
fn test_validate_url_input() {
    use crate::ui::add_modal::validate_url_input;

    // 1. 正常补全与提取
    let (url1, title1) = validate_url_input("github.com").unwrap();
    assert_eq!(url1, "https://github.com");
    assert_eq!(title1, "github.com");

    let (url2, title2) = validate_url_input("http://crates.io/crates/gpui").unwrap();
    assert_eq!(url2, "http://crates.io/crates/gpui");
    assert_eq!(title2, "crates.io");

    let (url3, _) = validate_url_input("localhost:8080").unwrap();
    assert_eq!(url3, "https://localhost:8080");

    // 2. 格式非法情况
    assert!(validate_url_input("").is_err());
    assert!(validate_url_input("   ").is_err());
    assert!(validate_url_input("bad url with spaces").is_err());
    assert!(validate_url_input("justaword").is_err());
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
    assert!(
        duration.as_millis() < 100,
        "20000次文件类型解析应在100ms内完成"
    );
}
