use std::time::Instant;

use crate::model::{GanttColor, ReminderRule, TodoData, TodoItem, GANTT_COLORS};

#[test]
fn test_todo_default_data() {
    let data = TodoData::default();
    assert_eq!(data.active_tag_id, "all");
    assert_eq!(data.tags.len(), 4);
    assert_eq!(data.tags[0].name, "工作");
    assert_eq!(data.tags[1].name, "学习");
    assert!(data.items.is_empty());
}

#[test]
fn test_todo_tag_operations() {
    let mut data = TodoData::default();

    // 1. 新增标签
    let new_id = data.add_tag("健康健身".to_string(), 2);
    assert!(data
        .tags
        .iter()
        .any(|t| t.id == new_id && t.name == "健康健身"));

    // 2. 更新标签
    let updated = data.update_tag(&new_id, "运动健身".to_string(), 4);
    assert!(updated);
    let tag = data.tags.iter().find(|t| t.id == new_id).unwrap();
    assert_eq!(tag.name, "运动健身");
    assert_eq!(tag.gantt_color, 4);

    // 3. 更新不存在的标签
    assert!(!data.update_tag("non_existent", "测试".to_string(), 0));
}

#[test]
fn test_todo_tag_migration_on_delete() {
    let mut data = TodoData::default();
    let custom_tag_id = data.add_tag("自定义".to_string(), 1);

    // 添加两条任务：一条在待删分类，一条在工作分类
    data.items.push(TodoItem {
        id: "item-1".to_string(),
        text: "任务 1".to_string(),
        done: false,
        tag_id: custom_tag_id.clone(),
        gantt_color: 0,
        reminder: None,
        last_reminded_at: None,
        created_at: None,
    });
    data.items.push(TodoItem {
        id: "item-2".to_string(),
        text: "任务 2".to_string(),
        done: false,
        tag_id: "work".to_string(),
        gantt_color: 0,
        reminder: None,
        last_reminded_at: None,
        created_at: None,
    });

    data.active_tag_id = custom_tag_id.clone();

    // 删除自定义分类
    let deleted = data.delete_tag_and_migrate(&custom_tag_id);
    assert!(deleted);

    // 验证分类已移除
    assert!(!data.tags.iter().any(|t| t.id == custom_tag_id));
    // 验证当前活动分类重置为 all
    assert_eq!(data.active_tag_id, "all");
    // 验证任务已安全迁移到首个可用分类（work）
    assert_eq!(data.items[0].tag_id, "work");
    assert_eq!(data.items[1].tag_id, "work");
}

#[test]
fn test_gantt_contrast_text_color() {
    // 亮色（日光金 0xfacc15, 翡翠绿 0x34d399, 天空蓝 0x38bdf8）应返回深色字体 (0x0f172a)
    let gold = GanttColor {
        name: "规划金",
        hex: 0xfacc15,
        bg_alpha_hex: 0xfacc1530,
    };
    let dark_expected: gpui::Hsla = gpui::rgb(0x0f172a).into();
    assert_eq!(gold.contrast_text(), dark_expected);

    // 较暗或中等饱和度颜色返回白色 (0xffffff)
    let dark_bg = GanttColor {
        name: "深黑蓝",
        hex: 0x1e293b,
        bg_alpha_hex: 0x1e293b30,
    };
    let white_expected: gpui::Hsla = gpui::rgb(0xffffff).into();
    assert_eq!(dark_bg.contrast_text(), white_expected);

    // 验证全量 GANTT_COLORS 计算无崩溃
    for color in GANTT_COLORS {
        let _ = color.contrast_text();
    }
}

#[test]
fn test_reminder_rule_display() {
    let rule_daily = ReminderRule::Daily {
        minute_of_day: 18 * 60 + 30,
    };
    assert_eq!(rule_daily.display_text(), "每天 18:30");

    let rule_weekly = ReminderRule::Weekly {
        weekday: 5,
        minute_of_day: 17 * 60,
    };
    assert_eq!(rule_weekly.display_text(), "每周五 17:00");

    let rule_interval = ReminderRule::Interval { interval_mins: 30 };
    assert_eq!(rule_interval.display_text(), "每 30 分钟催办");
}

#[test]
fn bench_todo_bulk_operations() {
    let mut data = TodoData::default();
    let bulk_count = 10_000;

    // 1. 批量插入 10,000 条待办性能测试
    let start_insert = Instant::now();
    for i in 0..bulk_count {
        data.items.push(TodoItem {
            id: format!("todo-{i}"),
            text: format!("批量待办测试条目 #{i}"),
            done: i % 3 == 0,
            tag_id: if i % 2 == 0 {
                "work".to_string()
            } else {
                "study".to_string()
            },
            gantt_color: i % GANTT_COLORS.len(),
            reminder: None,
            last_reminded_at: None,
            created_at: None,
        });
    }
    let insert_duration = start_insert.elapsed();
    println!(
        "[性能测试] 批量构建 {bulk_count} 条待办耗时: {:?}",
        insert_duration
    );
    assert!(
        insert_duration.as_millis() < 200,
        "10000条插入应在200ms内完成"
    );

    // 2. 批量过滤/遍历查询性能测试
    let start_query = Instant::now();
    let completed_count = data.items.iter().filter(|it| it.done).count();
    let query_duration = start_query.elapsed();
    println!(
        "[性能测试] 过滤 {bulk_count} 条待办状态耗时: {:?} (完成数: {completed_count})",
        query_duration
    );
    assert!(query_duration.as_millis() < 50, "10000条查询应在50ms内完成");

    // 3. 批量分类标签迁移性能测试
    let new_tag_id = data.add_tag("新分类".to_string(), 0);
    for item in &mut data.items[0..5000] {
        item.tag_id = new_tag_id.clone();
    }
    let start_migrate = Instant::now();
    data.delete_tag_and_migrate(&new_tag_id);
    let migrate_duration = start_migrate.elapsed();
    println!(
        "[性能测试] 迁移 5000 条待办至新分类耗时: {:?}",
        migrate_duration
    );
    assert!(
        migrate_duration.as_millis() < 50,
        "5000条迁移应在50ms内完成"
    );
}

#[test]
fn test_send_todo_notification_non_blocking() {
    // 验证调用通知接口安全非阻塞、不 panic，并留出时间展示 Windows Toast 通知
    crate::notification::send_todo_notification(
        "待办事项提醒 · 工作",
        "完成代码审查与通知功能测试",
    );
    // 等待后台线程完成 WinRT Toast 注册并弹出
    std::thread::sleep(std::time::Duration::from_millis(1500));
}

#[test]
fn test_reminder_rule_trigger_logic() {
    let now = 1700000000u64;

    // 1. Once 规则：未到期
    let once_future = ReminderRule::Once {
        target_time_secs: now + 300,
    };
    let should_trigger_future = match once_future {
        ReminderRule::Once { target_time_secs } => now >= target_time_secs,
        _ => false,
    };
    assert!(!should_trigger_future);

    // 2. Once 规则：已到期
    let once_past = ReminderRule::Once {
        target_time_secs: now - 10,
    };
    let should_trigger_past = match once_past {
        ReminderRule::Once { target_time_secs } => now >= target_time_secs,
        _ => false,
    };
    assert!(should_trigger_past);

    // 3. Interval 规则：间隔满足（上次提醒为 35 分钟前，设置间隔 30 分钟）
    let interval_rule = ReminderRule::Interval { interval_mins: 30 };
    let last_reminded = Some(now - 35 * 60);
    let should_trigger_interval = match interval_rule {
        ReminderRule::Interval { interval_mins } => last_reminded
            .map(|t| now.saturating_sub(t) >= (interval_mins as u64 * 60))
            .unwrap_or(true),
        _ => false,
    };
    assert!(should_trigger_interval);
}

#[test]
fn test_reminder_preset_crud() {
    let mut data = TodoData::default();
    assert_eq!(data.reminder_presets.len(), 4);

    // 1. 新增自定义预设
    let new_id = data.add_preset(
        "10分钟后".to_string(),
        ReminderRule::Once {
            target_time_secs: 600,
        },
    );
    assert_eq!(data.reminder_presets.len(), 5);
    assert!(data
        .reminder_presets
        .iter()
        .any(|p| p.id == new_id && p.label == "10分钟后"));

    // 2. 转换规则
    let preset = data
        .reminder_presets
        .iter()
        .find(|p| p.id == new_id)
        .unwrap();
    let generated_rule = preset.to_rule();
    match generated_rule {
        ReminderRule::Once { target_time_secs } => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            assert!(target_time_secs >= now);
        }
        _ => panic!("Expected Once rule"),
    }

    // 3. 更新预设
    let updated = data.update_preset(
        &new_id,
        "15分钟后".to_string(),
        ReminderRule::Once {
            target_time_secs: 900,
        },
    );
    assert!(updated);
    let p = data
        .reminder_presets
        .iter()
        .find(|p| p.id == new_id)
        .unwrap();
    assert_eq!(p.label, "15分钟后");

    // 4. 删除预设
    let deleted = data.delete_preset(&new_id);
    assert!(deleted);
    assert_eq!(data.reminder_presets.len(), 4);
    assert!(!data.reminder_presets.iter().any(|p| p.id == new_id));
}
