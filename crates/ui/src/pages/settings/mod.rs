pub mod about;
pub mod general;
pub mod menu;
pub mod shortcuts;
pub mod update_card;

pub use menu::SettingsTab;

use gpui::*;

pub fn render_settings_page(
    settings_tab: SettingsTab,
    cx: &mut Context<crate::main_window::MainWindow>,
) -> impl IntoElement {
    let right_content = match settings_tab {
        SettingsTab::General => general::render_general_settings(cx),
        SettingsTab::Shortcuts => shortcuts::render_shortcuts_settings(),
        SettingsTab::About => about::render_about_settings(cx, true),
        SettingsTab::Update => vec![
            div()
                .flex()
                .justify_between()
                .items_center()
                .w_full()
                .child(crate::layout::page_header(
                    "软件更新",
                    "检查新版本与配置自动更新",
                ))
                .into_any_element(),
            update_card::render_update_section(cx).into_any_element(),
        ],
    };

    div()
        .flex()
        .flex_1()
        .size_full()
        .overflow_hidden()
        .child(menu::render_settings_menu(settings_tab, cx))
        .child(
            div()
                .id("settings-page-scroll")
                .flex_1()
                .h_full()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .p(px(24.0))
                .gap(px(20.0))
                .children(right_content),
        )
}
