use std::backtrace::Backtrace;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// 初始化崩溃捕获机制
///
/// 包括：
/// 1. 设置 Rust 标准 panic hook，捕获未处理的 panic 并写入本地崩溃日志文件。
/// 2. 设置 Windows 未捕获原生异常过滤器（UnhandledExceptionFilter），捕获底层硬件/系统级崩溃。
pub fn init_crash_handler() {
    // 1. 设置 Rust Panic Hook
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let timestamp = get_current_timestamp();
        let file_timestamp = get_current_timestamp_for_filename();

        // 提取崩溃详细信息
        let payload_msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "未知 Panic 载荷".to_string()
        };

        let location_str = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "未知源码位置".to_string()
        };

        let current_thread = std::thread::current();
        let thread_name = current_thread.name().unwrap_or("<未命名线程>");

        // 强制捕获完整调用栈回溯
        let backtrace = Backtrace::force_capture();

        // 组装格式化崩溃报告
        let report = format!(
            "================================================================================\n\
             WIDGET-RS 崩溃报告 / CRASH REPORT\n\
             ================================================================================\n\
             崩溃时间 / Timestamp : {}\n\
             软件版本 / Version   : v{}\n\
             系统架构 / Platform  : {} {}\n\
             崩溃线程 / Thread    : {}\n\
             崩溃位置 / Location  : {}\n\
             崩溃原因 / Message   : {}\n\
             \n\
             --------------------------------------------------------------------------------\n\
             调用栈回溯 / Stack Backtrace:\n\
             --------------------------------------------------------------------------------\n\
             {}\n\
             ================================================================================\n",
            timestamp,
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            thread_name,
            location_str,
            payload_msg,
            backtrace
        );

        // 写入本地日志文件
        let saved_paths = write_crash_log(&file_timestamp, &report);

        // 在控制台输出（如果存在）
        eprintln!("{}", report);

        // 在 Windows 上弹出原生错误对话框提示用户
        show_crash_dialog(&payload_msg, &saved_paths);

        // 调用默认 hook（便于终端环境退出与信号传递）
        default_hook(panic_info);
    }));

    // 2. 设置 Windows SEH 原生异常捕获
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::System::Diagnostics::Debug::SetUnhandledExceptionFilter(Some(
            unhandled_exception_filter,
        ));
    }
}

/// Windows 未处理异常处理回调（捕获例如 0xC0000005 等原生系统崩溃）
#[cfg(windows)]
unsafe extern "system" fn unhandled_exception_filter(
    exception_info: *const windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> i32 {
    let timestamp = get_current_timestamp();
    let file_timestamp = get_current_timestamp_for_filename();

    let mut code: u32 = 0;
    let mut address: usize = 0;

    if !exception_info.is_null() {
        let record = (*exception_info).ExceptionRecord;
        if !record.is_null() {
            code = (*record).ExceptionCode as u32;
            address = (*record).ExceptionAddress as usize;
        }
    }

    let code_desc = match code {
        0xC0000005 => "STATUS_ACCESS_VIOLATION (内存非法访问 / 空指针解引用)",
        0xC000001D => "STATUS_ILLEGAL_INSTRUCTION (非法指令)",
        0xC000008C => "STATUS_ARRAY_BOUNDS_EXCEEDED (数组越界)",
        0xC000008D => "STATUS_FLOAT_DENORMAL_OPERAND (浮点操作数异常)",
        0xC000008E => "STATUS_FLOAT_DIVIDE_BY_ZERO (浮点除以零)",
        0xC0000094 => "STATUS_INTEGER_DIVIDE_BY_ZERO (整数除以零)",
        0xC00000FD => "STATUS_STACK_OVERFLOW (栈溢出)",
        _ => "未知系统级原生异常",
    };

    let report = format!(
        "================================================================================\n\
         WIDGET-RS 系统底层原生崩溃报告 / NATIVE CRASH REPORT\n\
         ================================================================================\n\
         崩溃时间 / Timestamp : {}\n\
         软件版本 / Version   : v{}\n\
         系统架构 / Platform  : {} {}\n\
         异常代码 / Code      : {:#010X} ({})\n\
         异常地址 / Address   : {:#018X}\n\
         ================================================================================\n",
        timestamp,
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        code,
        code_desc,
        address
    );

    let saved_paths = write_crash_log(&file_timestamp, &report);
    let msg = format!("系统原生异常: {:#010X} ({})", code, code_desc);
    show_crash_dialog(&msg, &saved_paths);

    // 传递给系统默认异常处理程序 (0 为 EXCEPTION_CONTINUE_SEARCH)
    0
}

/// 获取崩溃日志主存储目录
pub fn get_crash_log_dir() -> PathBuf {
    widget_core::get_log_dir()
}

/// 将崩溃报告写入本地日志文件，同时写入最新日志与带时间戳的历史归档
fn write_crash_log(file_timestamp: &str, report: &str) -> Vec<PathBuf> {
    let mut saved_paths = Vec::new();
    let log_dir = get_crash_log_dir();

    // 1. 写入时间戳归档日志，例如 logs/crash_20260901_112233.log
    let archive_path = log_dir.join(format!("crash_{}.log", file_timestamp));
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&archive_path)
    {
        let _ = file.write_all(report.as_bytes());
        let _ = file.flush();
        saved_paths.push(archive_path);
    }

    // 2. 写入/覆盖最新崩溃日志 logs/crash.log
    let latest_path = log_dir.join("crash.log");
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&latest_path)
    {
        let _ = file.write_all(report.as_bytes());
        let _ = file.flush();
        saved_paths.push(latest_path);
    }

    // 3. 后备机制：如果以上写入全部失败，尝试在当前工作目录/exe 目录直接创建 crash.log
    if saved_paths.is_empty() {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("crash.log")
        {
            let _ = file.write_all(report.as_bytes());
            let _ = file.flush();
            saved_paths.push(PathBuf::from("crash.log"));
        }
    }

    saved_paths
}

/// 弹出原生 Windows 崩溃提示框
fn show_crash_dialog(cause: &str, paths: &[PathBuf]) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            MessageBoxW, MB_ICONERROR, MB_OK, MB_SETFOREGROUND, MB_TOPMOST,
        };

        let path_display = if paths.is_empty() {
            "写入本地日志失败".to_string()
        } else {
            paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        };

        let title_wide = to_wide_string("widget-rs - 程序异常退出");
        let text = format!(
            "widget-rs 遇到了不可恢复的异常并即将退出。\n\n\
             崩溃原因 / Cause:\n\
             {}\n\n\
             崩溃日志已保存至 / Crash Log Path:\n\
             {}\n\n\
             请将此日志提供给开发者以协助排查问题。",
            cause, path_display
        );
        let text_wide = to_wide_string(&text);

        unsafe {
            MessageBoxW(
                0,
                text_wide.as_ptr(),
                title_wide.as_ptr(),
                MB_OK | MB_ICONERROR | MB_TOPMOST | MB_SETFOREGROUND,
            );
        }
    }
}

#[cfg(windows)]
fn to_wide_string(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// 获取当前格式化时间（YYYY-MM-DD HH:MM:SS.mmm）
fn get_current_timestamp() -> String {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::SYSTEMTIME;
        use windows_sys::Win32::System::SystemInformation::GetLocalTime;
        let mut st = SYSTEMTIME {
            wYear: 0,
            wMonth: 0,
            wDayOfWeek: 0,
            wDay: 0,
            wHour: 0,
            wMinute: 0,
            wSecond: 0,
            wMilliseconds: 0,
        };
        unsafe {
            GetLocalTime(&mut st);
        }
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond, st.wMilliseconds
        )
    }
    #[cfg(not(windows))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("Unix Timestamp: {}", secs)
    }
}

/// 获取当前时间用于日志文件名（YYYYMMDD_HHMMSS）
fn get_current_timestamp_for_filename() -> String {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::SYSTEMTIME;
        use windows_sys::Win32::System::SystemInformation::GetLocalTime;
        let mut st = SYSTEMTIME {
            wYear: 0,
            wMonth: 0,
            wDayOfWeek: 0,
            wDay: 0,
            wHour: 0,
            wMinute: 0,
            wSecond: 0,
            wMilliseconds: 0,
        };
        unsafe {
            GetLocalTime(&mut st);
        }
        format!(
            "{:04}{:02}{:02}_{:02}{:02}{:02}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
        )
    }
    #[cfg(not(windows))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{}", secs)
    }
}
