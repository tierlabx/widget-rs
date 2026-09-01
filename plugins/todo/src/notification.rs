use notify_rust::Notification;

/// 异步发送待办提醒通知
///
/// 在独立线程中派发系统原生 Toast 通知，避免阻塞 GPUI 主渲染循环。
pub fn send_todo_notification(title: &str, body: &str) {
    let title_owned = title.to_string();
    let body_owned = body.to_string();

    std::thread::spawn(move || {
        let mut notification = Notification::new();
        notification
            .appname("widget-rs")
            .summary(&title_owned)
            .body(&body_owned)
            .sound_name("Default");

        #[cfg(target_os = "windows")]
        {
            // Windows 系统需要已注册的 AUMID，使用系统通用 AUMID 确保 Toast 弹窗正常展示
            notification.app_id(
                "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
            );
        }

        if let Err(_err) = notification.show() {
            #[cfg(target_os = "windows")]
            {
                send_windows_toast_fallback(&title_owned, &body_owned);
            }
            #[cfg(not(target_os = "windows"))]
            {
                eprintln!("[todo_plugin] 发送系统通知失败: {err}");
            }
        }
    });
}

#[cfg(target_os = "windows")]
fn send_windows_toast_fallback(title: &str, body: &str) {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // 替换 XML 特殊字符
    let safe_title = title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let safe_body = body
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let script = format!(
        "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null;\
         [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null;\
         $xml = New-Object Windows.Data.Xml.Dom.XmlDocument;\
         $xml.LoadXml('<toast><visual><binding template=\"ToastGeneric\"><text>{safe_title}</text><text>{safe_body}</text></binding></visual><audio src=\"ms-winsoundevent:Notification.Default\"/></toast>');\
         $toast = [Windows.UI.Notifications.ToastNotification]::new($xml);\
         [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('{{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}}\\WindowsPowerShell\\v1.0\\powershell.exe').Show($toast);"
    );

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let _ = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}
