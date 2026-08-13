use gpui::prelude::FluentBuilder;
use gpui::*;
use raw_window_handle::HasWindowHandle;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, GetWindowLongW, SetWindowLongPtrW, SetWindowLongW,
    SetWindowPos, GWLP_WNDPROC, GWL_EXSTYLE, GWL_STYLE, HWND_TOPMOST, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WINDOWPOS, WM_NCCALCSIZE, WM_WINDOWPOSCHANGING,
    WS_BORDER, WS_CAPTION, WS_EX_CLIENTEDGE, WS_EX_WINDOWEDGE, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
};

fn original_procs() -> &'static Mutex<HashMap<isize, isize>> {
    static PROCS: OnceLock<Mutex<HashMap<isize, isize>>> = OnceLock::new();
    PROCS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn forced_bounds() -> &'static Mutex<HashMap<isize, (i32, i32, i32, i32)>> {
    static BOUNDS: OnceLock<Mutex<HashMap<isize, (i32, i32, i32, i32)>>> = OnceLock::new();
    BOUNDS.get_or_init(|| Mutex::new(HashMap::new()))
}

unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCCALCSIZE {
        if wparam != 0 {
            // 当 wparam 为 TRUE 时，拦截并返回 0 强制让系统把整个窗口当做客户区，
            // 彻底消灭左右下的不可见缩放边框（Invisible resize border）带来的缝隙。
            return 0;
        }
    }

    if msg == WM_WINDOWPOSCHANGING {
        if let Some(&(x, y, w, h)) = forced_bounds().lock().unwrap().get(&(hwnd as isize)) {
            let wp = &mut *(lparam as *mut WINDOWPOS);
            // 恢复为严格贴合屏幕
            wp.x = x;
            wp.y = y;
            wp.cx = w;
            wp.cy = h;
            // 清除保持尺寸和位置的标记，确保应用我们指定的大小和位置
            wp.flags &= !(SWP_NOMOVE | SWP_NOSIZE);
        }
    }
    let orig = *original_procs()
        .lock()
        .unwrap()
        .get(&(hwnd as isize))
        .unwrap_or(&0);
    if orig != 0 {
        CallWindowProcW(Some(std::mem::transmute(orig)), hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

pub struct BreakOverlay {
    styled: bool,
    is_primary: bool,
    bounds: (i32, i32, i32, i32),
}

impl BreakOverlay {
    pub fn new(is_primary: bool, bounds: (i32, i32, i32, i32)) -> Self {
        Self {
            styled: false,
            is_primary,
            bounds,
        }
    }
}

impl Render for BreakOverlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 首次渲染：子类化窗口，拦截位置变化，彻底锁死到物理屏幕
        if !self.styled {
            self.styled = true;
            if let Ok(h) = window.window_handle() {
                if let raw_window_handle::RawWindowHandle::Win32(h) = h.as_raw() {
                    let hwnd = h.hwnd.get();
                    unsafe {
                        // 记录绑定的物理坐标
                        forced_bounds()
                            .lock()
                            .unwrap()
                            .insert(hwnd as isize, self.bounds);

                        // 注入自定义 Window Proc 拦截 WM_WINDOWPOSCHANGING
                        let old_proc = SetWindowLongPtrW(
                            hwnd,
                            GWLP_WNDPROC,
                            overlay_wnd_proc as *const () as isize,
                        );
                        if old_proc != 0 {
                            original_procs()
                                .lock()
                                .unwrap()
                                .insert(hwnd as isize, old_proc);
                        }

                        // 移除标题栏和边框样式，防止拖动
                        let style = GetWindowLongW(hwnd, GWL_STYLE);
                        SetWindowLongW(
                            hwnd,
                            GWL_STYLE,
                            (style
                                & !(WS_CAPTION as i32)
                                & !(WS_THICKFRAME as i32)
                                & !(WS_BORDER as i32)
                                & !(WS_SYSMENU as i32))
                                | WS_POPUP as i32,
                        );

                        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                        SetWindowLongW(
                            hwnd,
                            GWL_EXSTYLE,
                            ex_style & !(WS_EX_CLIENTEDGE as i32) & !(WS_EX_WINDOWEDGE as i32),
                        );

                        // 置顶，并强行改变位置到绑定的物理屏幕（触发一次 WM_WINDOWPOSCHANGING）
                        // 注意：必须包含 SWP_FRAMECHANGED 才能让 Windows 重新计算并去除边框造成的客户区缝隙
                        let (x, y, w, h) = self.bounds;
                        SetWindowPos(
                            hwnd,
                            HWND_TOPMOST,
                            x,
                            y,
                            w,
                            h,
                            SWP_NOACTIVATE | SWP_FRAMECHANGED,
                        );
                    }
                }
            }
        }

        // 监听休息结束信号：清理钩子并自毁
        let break_active = cx
            .try_global::<crate::StretchlyBreakActive>()
            .is_some_and(|g| g.0);
        if !break_active {
            if let Ok(h) = window.window_handle() {
                if let raw_window_handle::RawWindowHandle::Win32(h) = h.as_raw() {
                    let hwnd = h.hwnd.get();
                    unsafe {
                        forced_bounds().lock().unwrap().remove(&(hwnd as isize));
                        if let Some(orig) =
                            original_procs().lock().unwrap().remove(&(hwnd as isize))
                        {
                            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, orig);
                        }
                    }
                }
            }
            window.remove_window();
        }

        // 副屏只显示背景色
        if !self.is_primary {
            return div().size_full().bg(rgba(0x02050eb0u32));
        }

        // 从全局快照读取渲染数据
        let snap = cx
            .try_global::<crate::StretchlyBreakSnapshot>()
            .cloned()
            .unwrap_or_default();

        let accent_color = if snap.is_mini {
            rgb(0x34d399u32)
        } else {
            rgb(0xa78bfau32)
        };
        let accent_bg = if snap.is_mini {
            rgba(0x34d39920u32)
        } else {
            rgba(0xa78bfa20u32)
        };

        div()
            .size_full()
            .bg(rgba(0x02050eb0u32))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .absolute()
                    .w(px(600.0))
                    .h(px(600.0))
                    .rounded(px(300.0))
                    .bg(rgba(0x0d1f3510u32))
                    .flex_shrink_0(),
            )
            .child(
                div()
                    .w(px(560.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(20.0))
                    .px(px(48.0))
                    .py(px(44.0))
                    .bg(rgba(0x0a1628ccu32))
                    .rounded(px(28.0))
                    .border_1()
                    .border_color(rgba(0xffffff18u32))
                    // 标题区
                    .child(
                        div().flex().flex_col().items_center().gap(px(6.0)).child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .px(px(14.0))
                                .py(px(5.0))
                                .bg(accent_bg)
                                .rounded(px(20.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(accent_color)
                                        .child(snap.break_label),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgba(0xc8d4f070u32))
                                        .child(format!("· {}", snap.break_duration_label)),
                                ),
                        ),
                    )
                    // 大号倒计时
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xf1f5f9u32))
                                    .child(snap.time_str.clone()),
                            )
                            .child(
                                div()
                                    .w(px(280.0))
                                    .h(px(5.0))
                                    .rounded(px(3.0))
                                    .bg(rgba(0xffffff10u32))
                                    .child(
                                        div()
                                            .h_full()
                                            .rounded(px(3.0))
                                            .bg(accent_color)
                                            .w(relative(snap.progress)),
                                    ),
                            ),
                    )
                    // 休息建议
                    .child(
                        div()
                            .w_full()
                            .px(px(16.0))
                            .py(px(14.0))
                            .bg(rgba(0xffffff07u32))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(rgba(0xffffff0au32))
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgba(0x94a3b860u32))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("休息建议"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgba(0xcbd5e1b0u32))
                                    .text_center()
                                    .child(snap.tip.clone()),
                            ),
                    )
                    .child(div().w_full().h(px(1.0)).bg(rgba(0xffffff0au32)))
                    // 按钮行
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(10.0))
                            // 推迟
                            .when(snap.allow_postpone, |d| {
                                d.child(
                                    div()
                                        .px(px(18.0))
                                        .py(px(9.0))
                                        .rounded(px(8.0))
                                        .bg(rgba(0xffffff0cu32))
                                        .hover(|s| s.bg(rgba(0xffffff18u32)))
                                        .cursor_pointer()
                                        .text_sm()
                                        .text_color(rgba(0xc8d4f0a0u32))
                                        .id("postpone-break-btn")
                                        .on_click(cx.listener(|_, _: &ClickEvent, _, cx| {
                                            cx.set_global(crate::StretchlyOverlayRequest(Some(
                                                crate::StretchlyOverlayAction::Postpone,
                                            )));
                                        }))
                                        .child(format!("推迟 {} 分钟", snap.postpone_mins)),
                                )
                            })
                            // 结束休息
                            .when(snap.allow_skip, |d| {
                                d.child(
                                    div()
                                        .px(px(18.0))
                                        .py(px(9.0))
                                        .rounded(px(8.0))
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .id("skip-break-btn")
                                        .when(snap.skip_available, |d2| {
                                            d2.bg(accent_bg)
                                                .hover(|s| s.bg(rgba(0x34d39930u32)))
                                                .cursor_pointer()
                                                .text_color(accent_color)
                                                .on_click(cx.listener(
                                                    |_, _: &ClickEvent, _, cx| {
                                                        cx.set_global(
                                                            crate::StretchlyOverlayRequest(Some(
                                                                crate::StretchlyOverlayAction::Skip,
                                                            )),
                                                        );
                                                    },
                                                ))
                                        })
                                        .when(!snap.skip_available, |d2| {
                                            d2.bg(rgba(0xffffff08u32))
                                                .text_color(rgba(0xffffff35u32))
                                        })
                                        .child(snap.skip_label.clone()),
                                )
                            }),
                    ),
            )
    }
}
