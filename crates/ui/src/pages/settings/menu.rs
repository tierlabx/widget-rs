use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::IconName;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    General,
    Logs,
    Shortcuts,
    About,
    Update,
}

pub fn render_settings_menu(
    current_tab: SettingsTab,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> impl IntoElement {
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
        // 搜索框（美观占位）
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(10.0))
                .py(px(6.0))
                .bg(rgb(0x181818))
                .border_1()
                .border_color(rgb(0x3d3a39))
                .rounded(px(6.0))
                .child(
                    div()
                        .text_color(rgb(0x8b949e))
                        .child(gpui_component::Icon::new(IconName::Search)),
                )
                .child(div().text_xs().text_color(rgb(0x8b949e)).child("搜索...")),
        )
        // 菜单项列表
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                // 通用分组
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(group_header("通用"))
                        .child(menu_sub_item(
                            "启动与常规",
                            "settings-menu-general",
                            SettingsTab::General,
                            current_tab,
                            cx,
                        ))
                        .child(menu_sub_item(
                            "运行日志",
                            "settings-menu-logs",
                            SettingsTab::Logs,
                            current_tab,
                            cx,
                        )),
                )
                // 快捷键分组
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(group_header("快捷键"))
                        .child(menu_sub_item(
                            "操作指南",
                            "settings-menu-shortcuts",
                            SettingsTab::Shortcuts,
                            current_tab,
                            cx,
                        )),
                )
                // 关于分组
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(group_header("关于"))
                        .child(menu_sub_item(
                            "版本与免责",
                            "settings-menu-about",
                            SettingsTab::About,
                            current_tab,
                            cx,
                        ))
                        .child(menu_sub_item(
                            "软件更新",
                            "settings-menu-update",
                            SettingsTab::Update,
                            current_tab,
                            cx,
                        )),
                ),
        )
}

fn group_header(title: &'static str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .px(px(8.0))
        .py(px(4.0))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x8b949e))
                .child(title),
        )
        .child(
            div()
                .text_color(rgb(0x5a5f67))
                .child(gpui_component::Icon::new(IconName::ChevronDown)),
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
