use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::Input;
use gpui_component::IconName;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    General,
    Logs,
    Shortcuts,
    About,
    Update,
}

struct MenuItemDef {
    tab: SettingsTab,
    id: &'static str,
    label: &'static str,
    keywords: &'static [&'static str],
}

struct MenuGroupDef {
    title: &'static str,
    group_idx: usize,
    items: &'static [MenuItemDef],
}

const MENU_GROUPS: [MenuGroupDef; 3] = [
    MenuGroupDef {
        title: "通用",
        group_idx: 0,
        items: &[
            MenuItemDef {
                tab: SettingsTab::General,
                id: "settings-menu-general",
                label: "启动与常规",
                keywords: &["启动", "常规", "开机", "自启动", "general", "start", "boot"],
            },
            MenuItemDef {
                tab: SettingsTab::Logs,
                id: "settings-menu-logs",
                label: "运行日志",
                keywords: &["日志", "运行", "崩溃", "crash", "log", "logs"],
            },
        ],
    },
    MenuGroupDef {
        title: "快捷键",
        group_idx: 1,
        items: &[MenuItemDef {
            tab: SettingsTab::Shortcuts,
            id: "settings-menu-shortcuts",
            label: "操作指南",
            keywords: &[
                "快捷键",
                "操作",
                "指南",
                "快捷",
                "shortcut",
                "shortcuts",
                "key",
                "keys",
            ],
        }],
    },
    MenuGroupDef {
        title: "关于",
        group_idx: 2,
        items: &[
            MenuItemDef {
                tab: SettingsTab::About,
                id: "settings-menu-about",
                label: "版本与免责",
                keywords: &["关于", "版本", "免责", "协议", "声明", "about", "version"],
            },
            MenuItemDef {
                tab: SettingsTab::Update,
                id: "settings-menu-update",
                label: "软件更新",
                keywords: &["更新", "软件更新", "升级", "版本更新", "update", "upgrade"],
            },
        ],
    },
];

pub fn render_settings_menu(
    current_tab: SettingsTab,
    search_input: &Option<Entity<gpui_component::input::InputState>>,
    collapsed_groups: [bool; 3],
    anim_tokens: [u32; 3],
    cx: &mut Context<crate::main_window::MainWindow>,
) -> impl IntoElement {
    let search_input = search_input.clone();

    let query = search_input
        .as_ref()
        .map(|input| input.read(cx).value().trim().to_lowercase())
        .unwrap_or_default();
    let is_searching = !query.is_empty();

    let search_input_for_clear = search_input.clone();
    let clear_handler = cx.listener(move |_this, _: &ClickEvent, window, cx| {
        if search_input_for_clear.is_some() {
            let new_input = cx.new(|cx| {
                gpui_component::input::InputState::new(window, cx).placeholder("搜索设置...")
            });
            cx.subscribe(
                &new_input,
                |_this: &mut crate::main_window::MainWindow,
                 _input: Entity<gpui_component::input::InputState>,
                 event: &gpui_component::input::InputEvent,
                 cx| {
                    if let gpui_component::input::InputEvent::Change = event {
                        cx.notify();
                    }
                },
            )
            .detach();
            _this.settings_search_input = Some(new_input);
            cx.notify();
        }
    });

    let mut rendered_groups = Vec::new();
    let mut total_matches = 0;

    for group in &MENU_GROUPS {
        let matching_items: Vec<&MenuItemDef> = if is_searching {
            group
                .items
                .iter()
                .filter(|item| {
                    item.label.to_lowercase().contains(&query)
                        || item
                            .keywords
                            .iter()
                            .any(|k| k.to_lowercase().contains(&query))
                })
                .collect()
        } else {
            group.items.iter().collect()
        };

        if matching_items.is_empty() {
            continue;
        }

        total_matches += matching_items.len();
        let is_collapsed = if is_searching {
            false
        } else {
            collapsed_groups[group.group_idx]
        };
        let anim_token = anim_tokens[group.group_idx];
        let items_count = matching_items.len();
        let base_height = items_count as f32 * 36.0 + (items_count.saturating_sub(1)) as f32 * 2.0;

        let group_idx = group.group_idx;
        let toggle_handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
            this.settings_collapsed_groups[group_idx] = !this.settings_collapsed_groups[group_idx];
            this.settings_anim_tokens[group_idx] =
                this.settings_anim_tokens[group_idx].wrapping_add(1);
            cx.notify();
        });

        let rendered_items: Vec<gpui::AnyElement> = matching_items
            .into_iter()
            .map(|item| {
                menu_sub_item(item.label, item.id, item.tab, current_tab, cx).into_any_element()
            })
            .collect();

        let header = div()
            .id(ElementId::Name(
                format!("group-header-{}", group_idx).into(),
            ))
            .flex()
            .items_center()
            .justify_between()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xffffff08)))
            .on_click(toggle_handler)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0x8b949e))
                    .child(group.title),
            )
            .child(
                div()
                    .text_color(rgb(0x5a5f67))
                    .child(gpui_component::Icon::new(if is_collapsed {
                        IconName::ChevronRight
                    } else {
                        IconName::ChevronDown
                    })),
            );

        if is_searching {
            rendered_groups.push(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(header)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .children(rendered_items),
                    )
                    .into_any_element(),
            );
        } else {
            let container = div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .overflow_hidden()
                .children(rendered_items);

            let animated_container = if is_collapsed {
                container
                    .with_animation(
                        ElementId::Name(
                            format!("group-anim-close-{}-{}", group_idx, anim_token).into(),
                        ),
                        Animation::new(Duration::from_millis(200)).with_easing(gpui::ease_in_out),
                        move |el, delta| {
                            let progress = (1.0 - delta).clamp(0.0, 1.0);
                            el.max_h(px(base_height * progress)).opacity(progress)
                        },
                    )
                    .into_any_element()
            } else {
                container
                    .with_animation(
                        ElementId::Name(
                            format!("group-anim-open-{}-{}", group_idx, anim_token).into(),
                        ),
                        Animation::new(Duration::from_millis(200)).with_easing(gpui::ease_in_out),
                        move |el, delta| {
                            let progress = delta.clamp(0.0, 1.0);
                            el.max_h(px(base_height * progress)).opacity(progress)
                        },
                    )
                    .into_any_element()
            };

            rendered_groups.push(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(header)
                    .child(animated_container)
                    .into_any_element(),
            );
        }
    }

    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .w(px(190.0))
        .h_full()
        .bg(rgb(0x0e0e0e))
        .border_r_1()
        .border_color(rgb(0x2d2a29))
        .p(px(16.0))
        .gap(px(16.0))
        // 搜索输入框
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(8.0))
                .py(px(4.0))
                .bg(rgb(0x181818))
                .border_1()
                .border_color(rgb(0x3d3a39))
                .rounded(px(6.0))
                .child(
                    div()
                        .text_color(rgb(0x8b949e))
                        .child(gpui_component::Icon::new(IconName::Search)),
                )
                .child(div().flex_1().when_some(search_input, |d, input| {
                    d.child(Input::new(&input).appearance(false).bordered(false))
                }))
                .when(is_searching, |d| {
                    d.child(
                        div()
                            .id("clear-search-btn")
                            .cursor_pointer()
                            .p(px(2.0))
                            .rounded(px(4.0))
                            .text_color(rgb(0x8b949e))
                            .hover(|s| s.text_color(rgb(0xf2f2f2)).bg(rgba(0xffffff15)))
                            .on_click(clear_handler)
                            .child(gpui_component::Icon::new(IconName::Close)),
                    )
                }),
        )
        // 菜单项列表
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .children(rendered_groups)
                .when(is_searching && total_matches == 0, |d| {
                    d.child(
                        div()
                            .px(px(8.0))
                            .py(px(12.0))
                            .text_xs()
                            .text_color(rgb(0x8b949e))
                            .child("未找到匹配栏目"),
                    )
                }),
        )
}

fn menu_sub_item(
    label: &'static str,
    id: &'static str,
    tab: SettingsTab,
    current: SettingsTab,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> impl IntoElement {
    let active = tab == current;
    let handler = cx.listener(move |this, _: &ClickEvent, _, cx| {
        this.settings_tab = tab;
        cx.notify();
    });

    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .w_full()
        .px(px(12.0))
        .py(px(6.0))
        .rounded(px(6.0))
        .when(active, |d| {
            d.bg(rgba(0x00d9921a))
                .border_1()
                .border_color(rgba(0x00d99230))
        })
        .cursor_pointer()
        .hover(
            move |s: gpui::StyleRefinement| {
                if !active {
                    s.bg(rgba(0xffffff08))
                } else {
                    s
                }
            },
        )
        .on_click(handler)
        .child(
            div()
                .text_sm()
                .font_weight(if active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if active { rgb(0x00d992) } else { rgb(0xb8b3b0) })
                .child(label),
        )
}
