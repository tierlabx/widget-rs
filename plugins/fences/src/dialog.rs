use gpui::*;

use crate::model::{FenceItem, FencesModel};
use crate::view::FencesWidget;

/// 打开选中的文件或文件夹（使用原生 ShellExecute API，杜绝控制台终端闪烁）
pub fn launch_item(path: &str) {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let path_wide: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
        let op_wide: Vec<u16> = OsStr::new("open").encode_wide().chain(Some(0)).collect();

        unsafe {
            ShellExecuteW(
                0,
                op_wide.as_ptr(),
                path_wide.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
    }

    #[cfg(not(windows))]
    {
        let _ = open::that(path);
    }
}

/// 打开添加文件/文件夹的对话框
pub fn open_add_dialog(this_entity: WeakEntity<FencesWidget>, target_cat: usize, cx: &mut App) {
    cx.spawn(async move |async_cx| {
        let script = if target_cat == 1 {
            "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
             $f = New-Object System.Windows.Forms.FolderBrowserDialog; \
             $f.Description = '选择要收纳的文件夹'; \
             if($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ Write-Output $f.SelectedPath }"
        } else if target_cat == 0 {
            "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
             $f = New-Object System.Windows.Forms.OpenFileDialog; \
             $f.Title = '选择要收纳的程序或快捷方式'; \
             $f.Filter = '应用程序 (*.exe;*.lnk)|*.exe;*.lnk|所有文件 (*.*)|*.*'; \
             if($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ Write-Output $f.FileName }"
        } else {
            "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
             $f = New-Object System.Windows.Forms.OpenFileDialog; \
             $f.Title = '选择要收纳的文件'; \
             $f.Filter = '所有文件 (*.*)|*.*'; \
             if($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){ Write-Output $f.FileName }"
        };

        let mut cmd = std::process::Command::new("powershell");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW，杜绝黑框终端窗口
        }
        cmd.args(["-NoProfile", "-Command", script]);

        let output = cmd.output();

        if let Ok(out) = output {
            let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path_str.is_empty() {
                let path = std::path::Path::new(&path_str);
                let raw_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path_str.clone());
                let name = if raw_name.to_lowercase().ends_with(".lnk") {
                    raw_name[..raw_name.len() - 4].to_string()
                } else {
                    raw_name
                };
                let is_dir = path.is_dir();

                let is_file = !is_dir;
                let added_path = path_str.clone();
                let _ = async_cx.update(|cx| {
                    let _ = this_entity.update(cx, |this, cx| {
                        if let Some(cat) = this.data.categories.get_mut(target_cat) {
                            cat.collapsed = false;
                            if target_cat < this.expand_progress.len() {
                                this.expand_progress[target_cat] = 1.0;
                            }
                            cat.items.push(FenceItem {
                                name,
                                path: path_str,
                                is_dir,
                            });
                            FencesModel::save(&this.data, cx);
                            cx.notify();
                        }
                    });
                });

                if is_file {
                    let _ = crate::icon_extractor::get_or_extract_icon(&added_path);
                    let _ = async_cx.update(|cx| {
                        let _ = this_entity.update(cx, |_, cx| {
                            cx.notify();
                        });
                    });
                }
            }
        }
    })
    .detach();
}
