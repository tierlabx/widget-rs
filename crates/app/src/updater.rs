//! 独立更新助手模块 (Updater Helper)
//!
//! 当主程序触发重启更新时，主程序会以 `--update-helper` 模式拉起本模块作为独立辅助进程。
//! 本模块会安全等待主进程完全退出、释放所有文件句柄后，执行目标可执行文件的覆盖替换，
//! 并在完成后拉起新版本主程序。

use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
};

/// 解析参数并运行独立更新助手逻辑
pub fn run_helper(args: &[String]) {
    let mut wait_pid: Option<u32> = None;
    let mut source_path: Option<PathBuf> = None;
    let mut target_path: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--wait-pid" if i + 1 < args.len() => {
                wait_pid = args[i + 1].parse::<u32>().ok();
                i += 1;
            }
            "--source" if i + 1 < args.len() => {
                source_path = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--target" if i + 1 < args.len() => {
                target_path = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    let (source, target) = match (source_path, target_path) {
        (Some(s), Some(t)) => (s, t),
        _ => {
            eprintln!("[updater] 缺少必要的 --source 或 --target 参数");
            return;
        }
    };

    // 1. 等待主进程退出
    if let Some(pid) = wait_pid {
        wait_for_process_exit(pid, Duration::from_secs(15));
    }

    // 2. 重试执行文件替换覆盖
    if let Err(e) = replace_target_file(&source, &target, 15, Duration::from_millis(200)) {
        eprintln!("[updater] 文件替换失败: {}", e);
        return;
    }

    // 3. 拉起新版本主程序
    if let Err(e) = spawn_new_process(&target) {
        eprintln!("[updater] 启动新版本失败: {}", e);
    }
}

/// 等待指定 PID 进程完全退出
fn wait_for_process_exit(pid: u32, timeout: Duration) {
    #[cfg(windows)]
    unsafe {
        let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if handle != 0 {
            let timeout_ms = timeout.as_millis().min(u32::MAX as u128) as u32;
            let _ = WaitForSingleObject(handle, timeout_ms);
            let _ = CloseHandle(handle);
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (pid, timeout);
    }

    // 稍微缓冲等待操作系统文件句柄彻底释放
    sleep(Duration::from_millis(150));
}

/// 重试覆盖目标文件
fn replace_target_file(
    source: &Path,
    target: &Path,
    max_retries: usize,
    retry_interval: Duration,
) -> Result<(), String> {
    if !source.exists() {
        return Err(format!("更新源文件不存在: {:?}", source));
    }

    let mut last_err = String::new();
    for _ in 0..max_retries {
        match std::fs::copy(source, target) {
            Ok(_) => return Ok(()),
            Err(e) => {
                last_err = e.to_string();
                sleep(retry_interval);
            }
        }
    }

    Err(format!("超过最大重试次数: {}", last_err))
}

/// 拉起目标新版本程序
fn spawn_new_process(target: &Path) -> Result<(), String> {
    std::process::Command::new(target)
        .spawn()
        .map_err(|e| format!("启动新进程失败: {}", e))?;
    Ok(())
}
