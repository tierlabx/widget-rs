/// 初始化并向 Windows 注册当前进程的 AppUserModelID (AUMID)
///
/// 确保系统 Toast 原生通知与任务栏正确展示应用名称为 "桌面小部件 (widget-rs)"，
/// 而不是回退显示为 "Windows PowerShell" 或通用宿主。
pub fn init_app_user_model_id() {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        // 1. 设置当前进程的 Explicit AppUserModelID
        let app_id_wide: Vec<u16> = OsStr::new("tierlabx.widget-rs")
            .encode_wide()
            .chain(Some(0))
            .collect();
        unsafe {
            windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(
                app_id_wide.as_ptr(),
            );
        }

        // 2. 在 HKCU\Software\Classes\AppUserModelId\tierlabx.widget-rs 注册 DisplayName 与图标
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        if let Ok((key, _)) =
            hkcu.create_subkey("Software\\Classes\\AppUserModelId\\tierlabx.widget-rs")
        {
            let icon_path = widget_core::get_app_icon_path();
            let _ = key.set_value("IconUri", &icon_path.to_string_lossy().to_string());
            let _ = key.set_value("IconBackgroundColor", &"0");
        }
    }
}
