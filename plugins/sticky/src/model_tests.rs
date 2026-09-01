use std::time::Instant;

use crate::model::{StickyData, StickyNote, STICKY_THEMES};

#[test]
fn test_sticky_default_data() {
    let data = StickyData::default();
    assert_eq!(data.notes.len(), 1);
    assert_eq!(data.current_index, 0);
    assert_eq!(data.current().content, "");
    assert_eq!(data.current().color_index, 0);
}

#[test]
fn test_sticky_note_lifecycle() {
    let mut data = StickyData::default();
    data.current_mut().content = "第一张便签".to_string();
    data.current_mut().color_index = 2; // 粉色

    // 1. 新建便签（应继承当前便签颜色并切换到新便签）
    data.new_note();
    assert_eq!(data.notes.len(), 2);
    assert_eq!(data.current_index, 1);
    assert_eq!(data.current().color_index, 2);
    assert_eq!(data.current().content, "");

    data.current_mut().content = "第二张便签".to_string();

    // 2. 翻页测试
    data.prev();
    assert_eq!(data.current_index, 0);
    assert_eq!(data.current().content, "第一张便签");

    data.prev(); // 边界保护
    assert_eq!(data.current_index, 0);

    data.next();
    assert_eq!(data.current_index, 1);
    assert_eq!(data.current().content, "第二张便签");

    data.next(); // 边界保护
    assert_eq!(data.current_index, 1);

    // 3. 删除便签
    data.delete_current();
    assert_eq!(data.notes.len(), 1);
    assert_eq!(data.current_index, 0);
    assert_eq!(data.current().content, "第一张便签");

    // 4. 仅剩一张时删除（不移除卡片，只清空内容）
    data.delete_current();
    assert_eq!(data.notes.len(), 1);
    assert_eq!(data.current().content, "");
}

#[test]
fn test_sticky_themes() {
    assert_eq!(STICKY_THEMES.len(), 6);
    assert_eq!(STICKY_THEMES[0].name, "黄色");
    assert_eq!(STICKY_THEMES[5].name, "碳黑");
}

#[test]
fn bench_sticky_notes_lifecycle() {
    let mut data = StickyData::default();
    let note_count = 5_000;

    let start = Instant::now();
    for i in 0..note_count {
        data.notes.push(StickyNote {
            content: format!("便签内容测试 #{i}"),
            color_index: i % STICKY_THEMES.len(),
            images: vec![],
        });
    }
    let duration = start.elapsed();
    println!(
        "[性能测试] 批量构建 {note_count} 张便签耗时: {:?}",
        duration
    );
    assert!(duration.as_millis() < 50, "5000张便签构建应在50ms内完成");
}
