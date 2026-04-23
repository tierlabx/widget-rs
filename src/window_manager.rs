use crate::ui::{MainWindow, StickyWidget, TodoWidget};
use slint::ComponentHandle;

pub struct WindowManager {
    main_window: MainWindow,
    sticky_widget: StickyWidget,
    todo_widget: TodoWidget,
}

impl WindowManager {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let main_window   = MainWindow::new()?;
        let sticky_widget = StickyWidget::new()?;
        let todo_widget   = TodoWidget::new()?;

        main_window.window().set_position(slint::LogicalPosition::new(200.0, 150.0));
        sticky_widget.window().set_position(slint::LogicalPosition::new(50.0, 50.0));
        todo_widget.window().set_position(slint::LogicalPosition::new(400.0, 50.0));

        Ok(Self { main_window, sticky_widget, todo_widget })
    }

    pub fn main_window(&self)   -> &MainWindow   { &self.main_window   }
    #[allow(dead_code)]
    pub fn sticky_widget(&self) -> &StickyWidget { &self.sticky_widget }
    #[allow(dead_code)]
    pub fn todo_widget(&self)   -> &TodoWidget   { &self.todo_widget   }

    /// 显示所有窗口，并安排在事件循环启动后移除任务栏图标
    pub fn show_all(&self) {
        let _ = self.main_window.show();
        let _ = self.sticky_widget.show();
        let _ = self.todo_widget.show();
    }

    /// 在事件循环内（Timer 回调中）调用：使用 Weak 引用精确获取 HWND 后处理任务栏
    pub fn schedule_taskbar_fix(&self) {
        #[cfg(target_os = "windows")]
        {
            // 必须在 show() + event loop 启动后才能获取有效 HWND，用 Weak + Timer 实现
            let sticky_weak = self.sticky_widget.as_weak();
            let todo_weak   = self.todo_widget.as_weak();

            // 创建一个永久隐藏的 dummy owner 窗口（Win11 最可靠的任务栏隐藏方案）
            let dummy_hwnd = create_hidden_dummy_window();
            println!("[taskbar] dummy owner HWND={:#x}", dummy_hwnd);

            slint::Timer::single_shot(std::time::Duration::from_millis(500), move || {
                let sticky_hwnd = sticky_weak.upgrade()
                    .and_then(|c| get_hwnd(c.window()));
                let todo_hwnd = todo_weak.upgrade()
                    .and_then(|c| get_hwnd(c.window()));

                println!("[taskbar] sticky={:?} todo={:?}", sticky_hwnd, todo_hwnd);

                if let Some(h) = sticky_hwnd { set_dummy_owner_and_hide(h, dummy_hwnd); }
                if let Some(h) = todo_hwnd   { set_dummy_owner_and_hide(h, dummy_hwnd); }
            });
        }
    }
}

/// 通过 raw-window-handle 0.6 获取精确 HWND（只在事件循环启动后有效）
#[cfg(target_os = "windows")]
fn get_hwnd(window: &slint::Window) -> Option<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let slint_wh = window.window_handle();
    let raw_wh   = slint_wh.window_handle().ok()?;
    match raw_wh.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get() as isize),
        _ => None,
    }
}

/// 创建一个永远不显示的 OVERLAPPED 窗口，用作 dummy owner。
/// 拥有一个不可见 owner 的弹出窗口不会出现在 Win11 任务栏。
#[cfg(target_os = "windows")]
fn create_hidden_dummy_window() -> isize {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, WS_OVERLAPPED,
    };
    let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
    let title: Vec<u16> = "\0".encode_utf16().collect();
    unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED,
            0, 0, 1, 1,
            0, 0, 0,
            std::ptr::null(),
        )
    }
}

/// 以三层保障将弹出窗口从任务栏移除：
///   1. 设置 dummy 不可见 owner（GWLP_HWNDPARENT）
///   2. 设置 WS_EX_TOOLWINDOW，清除 WS_EX_APPWINDOW
///   3. hide → ShowNoActivate（强制 Win11 Shell 重新评估）
#[cfg(target_os = "windows")]
fn set_dummy_owner_and_hide(hwnd: isize, dummy_owner: isize) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, ShowWindow,
        GWL_EXSTYLE, GWLP_HWNDPARENT,
        WS_EX_TOOLWINDOW, WS_EX_APPWINDOW,
        SW_HIDE, SW_SHOWNOACTIVATE,
    };
    unsafe {
        // 1. 设置不可见 dummy 窗口为 owner
        SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, dummy_owner);

        // 2. 设置 WS_EX_TOOLWINDOW
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let new_ex = (ex & !(WS_EX_APPWINDOW as isize)) | WS_EX_TOOLWINDOW as isize;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex);

        // 3. 强制刷新任务栏
        ShowWindow(hwnd, SW_HIDE);
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        println!("[taskbar] HWND {:#x} hidden (dummy owner={:#x})", hwnd, dummy_owner);
    }
}
