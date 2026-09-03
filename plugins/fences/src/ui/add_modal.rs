use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};

/// 校验并规范化用户输入的网址
pub fn validate_url_input(raw: &str) -> Result<(String, String), &'static str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("网址不能为空");
    }

    // 自动补齐协议头
    let full_url = if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };

    let without_proto = full_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    let host = without_proto
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .split('#')
        .next()
        .unwrap_or("")
        .trim();

    if host.is_empty() || host.contains(' ') || host.contains('\t') {
        return Err("网址格式不正确，不能包含空格");
    }

    let host_name = host.split(':').next().unwrap_or(host);
    if host_name != "localhost" && !host_name.contains('.') {
        return Err("请输入完整的网址域名 (例如: github.com)");
    }

    let default_title = host_name.to_string();
    Ok((full_url, default_title))
}

/// 添加网页书签弹窗状态
pub struct AddUrlModalState {
    pub url_input: Entity<InputState>,
    pub name_input: Entity<InputState>,
    pub error_msg: Option<String>,
}

impl AddUrlModalState {
    pub fn new<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> Self {
        let url_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("输入网址 (如 github.com)..."));
        let name_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("书签名称 (选填，留空自动提取域名)...")
        });
        Self {
            url_input,
            name_input,
            error_msg: None,
        }
    }
}

/// 渲染添加网页书签 GPUI 原生浮层弹窗
pub fn render_add_url_modal<V: 'static>(
    modal: &AddUrlModalState,
    on_confirm: impl Fn(&mut V, &mut Window, &mut Context<V>, String, String) + 'static + Clone,
    on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_error: impl Fn(&mut V, &mut Context<V>, Option<String>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let url_input = modal.url_input.clone();
    let name_input = modal.name_input.clone();
    let error_msg = modal.error_msg.clone();
    let has_error = error_msg.is_some();

    let on_close_bg = on_close.clone();
    let on_close_btn = on_close.clone();

    let on_confirm_click = {
        let on_confirm = on_confirm.clone();
        let on_error = on_error.clone();
        let url_input = url_input.clone();
        let name_input = name_input.clone();
        cx.listener(move |this, _: &ClickEvent, window, cx| {
            let raw_url = url_input.read(cx).value().to_string();
            let raw_name = name_input.read(cx).value().to_string();

            match validate_url_input(&raw_url) {
                Ok((valid_url, default_title)) => {
                    let final_name = if raw_name.trim().is_empty() {
                        default_title
                    } else {
                        raw_name.trim().to_string()
                    };
                    on_error(this, cx, None);
                    on_confirm(this, window, cx, valid_url, final_name);
                }
                Err(err) => {
                    on_error(this, cx, Some(err.to_string()));
                }
            }
        })
    };

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        // 1. 半透明背景遮罩（点击关闭）
        .child(
            div()
                .absolute()
                .inset_0()
                .bg(rgba(0x00000088))
                .id("add-url-modal-backdrop")
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    on_close_bg(this, window, cx);
                })),
        )
        // 2. 居中卡片内容
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .w(px(280.0))
                .p(px(14.0))
                .gap(px(10.0))
                .bg(rgba(0x0a1526fc))
                .rounded(px(12.0))
                .border_1()
                .border_color(if has_error {
                    rgba(0xef444480)
                } else {
                    rgba(0x38bdf838)
                })
                .shadow_lg()
                .id("add-url-modal-card")
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                .on_click(|_, _, cx| cx.stop_propagation())
                // 标题与关闭按钮
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .text_color(rgb(0x38bdf8))
                                        .child(Icon::new(IconName::Globe).size(px(14.0))),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xf8fafc))
                                        .child("添加快捷网址"),
                                ),
                        )
                        .child(
                            div()
                                .w(px(20.0))
                                .h(px(20.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .text_color(rgba(0xffffff60))
                                .hover(|s| s.bg(rgba(0xffffff15)).text_color(rgb(0xffffff)))
                                .id("add-url-close-btn")
                                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                                    on_close_btn(this, window, cx);
                                }))
                                .child(Icon::new(IconName::Close).size(px(12.0))),
                        ),
                )
                // 网址输入框
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgba(0xffffff70))
                                        .child("网页链接"),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(rgba(0xffffff40))
                                        .child("支持自动补全 https://"),
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(if has_error {
                                    rgb(0xef4444)
                                } else {
                                    rgba(0x38bdf825)
                                })
                                .bg(rgba(0x00000040))
                                .p(px(2.0))
                                .child(Input::new(&url_input)),
                        ),
                )
                // 错误提示文案
                .children(error_msg.map(|err| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .text_size(px(11.0))
                        .text_color(rgb(0xf87171))
                        .child(Icon::new(IconName::Close).size(px(11.0)))
                        .child(err)
                }))
                // 名称输入框
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgba(0xffffff70))
                                .child("显示名称"),
                        )
                        .child(
                            div()
                                .w_full()
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(rgba(0x38bdf825))
                                .bg(rgba(0x00000040))
                                .p(px(2.0))
                                .child(Input::new(&name_input)),
                        ),
                )
                // 底部操作按钮
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(px(8.0))
                        .pt(px(4.0))
                        .child(
                            div()
                                .px(px(10.0))
                                .py(px(4.0))
                                .rounded(px(5.0))
                                .cursor_pointer()
                                .text_xs()
                                .text_color(rgba(0xffffff80))
                                .bg(rgba(0xffffff10))
                                .hover(|s| s.bg(rgba(0xffffff20)).text_color(rgb(0xffffff)))
                                .id("add-url-cancel-btn")
                                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                                    on_close(this, window, cx);
                                }))
                                .child("取消"),
                        )
                        .child(
                            div()
                                .px(px(12.0))
                                .py(px(4.0))
                                .rounded(px(5.0))
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .bg(rgb(0x0284c7))
                                .hover(|s| s.bg(rgb(0x0369a1)))
                                .id("add-url-confirm-btn")
                                .on_click(on_confirm_click)
                                .child("添加"),
                        ),
                ),
        )
}
