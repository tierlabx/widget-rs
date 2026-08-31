use super::update_card::render_update_section;
use crate::layout::{page_header, settings_card};
use gpui::*;
use gpui_component::IconName;

pub fn render_about_settings(
    cx: &mut Context<crate::main_window::MainWindow>,
    include_header: bool,
) -> Vec<gpui::AnyElement> {
    let mut elements = Vec::new();

    if include_header {
        elements.push(
            div()
                .flex()
                .justify_between()
                .items_center()
                .w_full()
                .child(page_header("关于", "关于 Widget-RS、开源协议及软件更新"))
                .into_any_element(),
        );
    }

    // 关于信息卡片
    elements.push(
        settings_card()
            .p(px(24.0))
            .gap(px(20.0))
            // 版本
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xf2f2f2))
                            .child("版本:"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0x8b949e))
                            .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                    ),
            )
            // 开源地址与操作按钮
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xf2f2f2))
                            .child("开源地址:"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0x00d992))
                            .child("https://github.com/tierlabx/widget-rs"),
                    )
                    // 复制按钮
                    .child(
                        div()
                            .id("copy-repo-url-btn")
                            .cursor_pointer()
                            .p(px(4.0))
                            .rounded(px(4.0))
                            .text_color(rgb(0x8b949e))
                            .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0xf2f2f2)))
                            .on_click(|_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(
                                    "https://github.com/tierlabx/widget-rs".to_string(),
                                ));
                            })
                            .child(gpui_component::Icon::new(IconName::Copy)),
                    )
                    // 打开浏览器按钮
                    .child(
                        div()
                            .id("open-repo-url-btn")
                            .cursor_pointer()
                            .p(px(4.0))
                            .rounded(px(4.0))
                            .text_color(rgb(0x8b949e))
                            .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0xf2f2f2)))
                            .on_click(|_, _, _| {
                                let _ = open::that("https://github.com/tierlabx/widget-rs");
                            })
                            .child(gpui_component::Icon::new(IconName::ExternalLink)),
                    ),
            )
            // 免责声明 (测试版)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .pt(px(8.0))
                    .border_t_1()
                    .border_color(rgb(0x252525))
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xf2f2f2))
                            .child("免责声明 (测试版)"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x8b949e))
                            .line_height(relative(1.5))
                            .child(
                                "本软件目前为开源测试版本，非稳定商业版，可能存在未知 Bug、兼容性问题、连接异常或潜在安全风险。",
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .text_sm()
                            .text_color(rgb(0x8b949e))
                            .line_height(relative(1.5))
                            .child("1. 本软件按“现状”提供，无任何明示或默示担保，包括但不限于稳定性、安全性、适用性。")
                            .child("2. 你自行承担使用本软件导致的一切风险，包括但不限于：数据丢失、服务中断、连接异常、信息泄露、业务损失。")
                            .child("3. 本软件仅用于你有权管理的合法环境，禁止用于未授权访问。")
                            .child("4. 作者与贡献者不对任何直接或间接损失承担法律责任。")
                            .child("5. 使用即表示你已阅读、理解并完全同意本声明。"),
                    ),
            )
            // 数据与安全提示
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .pt(px(8.0))
                    .border_t_1()
                    .border_color(rgb(0x252525))
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xf2f2f2))
                            .child("数据与安全提示"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .text_sm()
                            .text_color(rgb(0x8b949e))
                            .line_height(relative(1.5))
                            .child("· 请自行备份重要数据与连接凭证。")
                            .child("· 生产环境谨慎使用，建议先在测试环境验证。")
                            .child("· 敏感信息（密码、密钥）请自行评估存储风险。"),
                    ),
            )
            .into_any_element(),
    );

    // 更新卡片
    elements.push(render_update_section(cx).into_any_element());

    elements
}
