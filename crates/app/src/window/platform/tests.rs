use super::windows::should_block_sibling_zorder_change;
use windows_sys::Win32::UI::WindowsAndMessaging::{HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOZORDER};

const WIDGET_A: isize = 0x1001;
const WIDGET_B: isize = 0x1002;
const DESKTOP_HOST: isize = 0x2001;
const EXTERNAL_APP: isize = 0x3001;

fn is_widget(hwnd: isize) -> bool {
    hwnd == WIDGET_A || hwnd == WIDGET_B
}

#[test]
fn win_d_raise_desktop_allows_zorder_change() {
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A,
        0,
        0,
        false,
        0,
        DESKTOP_HOST,
        false,
        is_widget,
    ));
}

#[test]
fn external_app_foreground_allows_zorder_change() {
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A,
        0,
        0,
        false,
        0,
        EXTERNAL_APP,
        false,
        is_widget,
    ));
}

#[test]
fn clicking_sibling_widget_blocks_zorder_linkage() {
    assert!(should_block_sibling_zorder_change(
        WIDGET_A, 0, 0, false, WIDGET_B, WIDGET_B, false, is_widget,
    ));
}

#[test]
fn clicking_self_widget_allows_zorder_change() {
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A, 0, 0, false, WIDGET_A, WIDGET_A, false, is_widget,
    ));
}

#[test]
fn stale_active_sibling_does_not_block_when_desktop_is_foreground() {
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A,
        0,
        0,
        false,
        WIDGET_B,
        DESKTOP_HOST,
        false,
        is_widget,
    ));
}

#[test]
fn zero_foreground_falls_back_to_active_sibling() {
    assert!(should_block_sibling_zorder_change(
        WIDGET_A, 0, 0, false, WIDGET_B, 0, false, is_widget,
    ));
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A, 0, 0, false, 0, 0, false, is_widget,
    ));
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A,
        0,
        0,
        false,
        EXTERNAL_APP,
        0,
        false,
        is_widget,
    ));
}

#[test]
fn topmost_explicit_and_nozorder_flags_are_never_blocked() {
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A,
        SWP_NOZORDER,
        0,
        false,
        WIDGET_B,
        WIDGET_B,
        false,
        is_widget,
    ));
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A,
        0,
        HWND_TOPMOST,
        false,
        WIDGET_B,
        WIDGET_B,
        false,
        is_widget,
    ));
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A,
        0,
        HWND_NOTOPMOST,
        false,
        WIDGET_B,
        WIDGET_B,
        false,
        is_widget,
    ));
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A, 0, 0, true, WIDGET_B, WIDGET_B, false, is_widget,
    ));
    assert!(!should_block_sibling_zorder_change(
        WIDGET_A, 0, 0, false, WIDGET_B, WIDGET_B, true, is_widget,
    ));
}
