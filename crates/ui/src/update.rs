use gpui::*;
use serde_json::Value;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// 更新检查状态
#[derive(Clone, Debug)]
pub enum UpdateStatus {
    /// 初始状态
    Idle,
    /// 正在检查中
    Checking,
    /// 发现新版本
    Available {
        version: String,
        download_url: String,
        release_notes: String,
        is_installer: bool,
    },
    /// 正在下载 (0-100%)
    Downloading(u8),
    /// 下载并准备就绪，待重启应用
    ReadyToRestart {
        new_exe_path: PathBuf,
        is_installer: bool,
    },
    /// 已是最新版本
    UpToDate,
    /// 错误
    Error(String),
}

pub struct MainWindowUpdateBridge {
    pub status: UpdateStatus,
    /// 用户是否已手动关闭本次弹窗提醒
    pub dismissed: bool,
}

impl Global for MainWindowUpdateBridge {}

/// 关闭更新提醒弹窗
pub fn dismiss_update_modal(cx: &mut App) {
    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
        bridge.dismissed = true;
    });
    cx.refresh_windows();
}

/// 检查新版本
pub fn check_for_update(cx: &mut App) {
    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
        bridge.status = UpdateStatus::Checking;
        bridge.dismissed = false;
    });
    cx.refresh_windows();

    let async_cx = cx.to_async();
    cx.foreground_executor()
        .spawn(async move {
            let status = async_cx
                .background_executor()
                .spawn(async move {
                    let resp = match ureq::get(
                        "https://api.github.com/repos/tierlabx/widget-rs/releases/latest",
                    )
                    .header("User-Agent", "widget-rs-updater")
                    .header("Accept", "application/vnd.github.v3+json")
                    .call()
                    {
                        Ok(r) => r,
                        Err(e) => return UpdateStatus::Error(format!("网络请求失败: {}", e)),
                    };

                    let body: Value = match resp.into_body().read_json() {
                        Ok(b) => b,
                        Err(e) => return UpdateStatus::Error(format!("解析响应失败: {}", e)),
                    };

                    let tag_name = match body["tag_name"].as_str() {
                        Some(t) => t,
                        None => return UpdateStatus::Error("无法获取版本号".to_string()),
                    };

                    let release_notes = body["body"]
                        .as_str()
                        .unwrap_or("包含性能优化与已知问题修复。")
                        .to_string();

                    let current_version = env!("CARGO_PKG_VERSION");
                    let latest_version = tag_name.trim_start_matches('v');

                    if latest_version != current_version {
                        let mut zip_url = None;
                        let mut setup_url = None;

                        if let Some(assets) = body["assets"].as_array() {
                            for asset in assets {
                                if let (Some(name), Some(url)) = (
                                    asset["name"].as_str(),
                                    asset["browser_download_url"].as_str(),
                                ) {
                                    if name.ends_with(".zip")
                                        && (name.contains("windows")
                                            || name.contains("x86_64")
                                            || name.contains("widget-rs"))
                                    {
                                        zip_url = Some(url.to_string());
                                        break;
                                    } else if name.ends_with(".exe") && name.contains("setup") {
                                        setup_url = Some(url.to_string());
                                    }
                                }
                            }
                        }

                        // 优先选择 zip 免安装热更新包，若无则降级使用 setup 安装器
                        if let Some(download_url) = zip_url {
                            UpdateStatus::Available {
                                version: latest_version.to_string(),
                                download_url,
                                release_notes,
                                is_installer: false,
                            }
                        } else if let Some(download_url) = setup_url {
                            UpdateStatus::Available {
                                version: latest_version.to_string(),
                                download_url,
                                release_notes,
                                is_installer: true,
                            }
                        } else {
                            UpdateStatus::UpToDate
                        }
                    } else {
                        UpdateStatus::UpToDate
                    }
                })
                .await;

            let _ = async_cx.update(|cx| {
                cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
                    bridge.status = status;
                });
                cx.refresh_windows();
            });
        })
        .detach();
}

/// 流式分块下载并在下载后自动解压准备就绪
pub fn download_update(url: String, is_installer: bool, cx: &mut App) {
    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
        bridge.status = UpdateStatus::Downloading(0);
        bridge.dismissed = false;
    });
    cx.refresh_windows();

    let (progress_tx, progress_rx) = std::sync::mpsc::channel::<Option<UpdateStatus>>();
    let async_cx = cx.to_async();

    // 后台独立执行下载与解压
    cx.background_executor()
        .spawn(async move {
            let status = (|| {
                let resp = match ureq::get(&url)
                    .header("User-Agent", "widget-rs-updater")
                    .call()
                {
                    Ok(r) => r,
                    Err(e) => return UpdateStatus::Error(format!("下载失败: {}", e)),
                };

                let content_length: Option<u64> = resp
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());

                let temp_dir = std::env::temp_dir().join("widget-rs-update");
                let _ = std::fs::create_dir_all(&temp_dir);

                let file_name = url
                    .split('/')
                    .next_back()
                    .unwrap_or(if is_installer {
                        "widget-rs-setup.exe"
                    } else {
                        "widget-rs.zip"
                    })
                    .to_string();
                let dest = temp_dir.join(&file_name);

                let mut file = match std::fs::File::create(&dest) {
                    Ok(f) => f,
                    Err(e) => return UpdateStatus::Error(format!("创建文件失败: {}", e)),
                };

                let mut reader = resp.into_body().into_reader();
                let mut buffer = [0u8; 64 * 1024]; // 64KB 缓冲区
                let mut downloaded_bytes: u64 = 0;
                let mut last_percent: u8 = 0;

                loop {
                    let bytes_read = match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) => return UpdateStatus::Error(format!("读取下载流失败: {}", e)),
                    };

                    if let Err(e) = file.write_all(&buffer[..bytes_read]) {
                        return UpdateStatus::Error(format!("写入临时文件失败: {}", e));
                    }

                    downloaded_bytes += bytes_read as u64;

                    if let Some(total) = content_length {
                        if total > 0 {
                            let percent =
                                ((downloaded_bytes as f64 / total as f64) * 100.0).min(99.0) as u8;
                            if percent > last_percent {
                                last_percent = percent;
                                let _ = progress_tx.send(Some(UpdateStatus::Downloading(percent)));
                            }
                        }
                    }
                }

                // 若为 zip 压缩包，则自动解压提取出 widget-rs.exe
                if !is_installer && dest.extension().is_some_and(|ext| ext == "zip") {
                    match extract_zip_update(&dest, &temp_dir) {
                        Ok(exe_path) => UpdateStatus::ReadyToRestart {
                            new_exe_path: exe_path,
                            is_installer: false,
                        },
                        Err(err) => UpdateStatus::Error(format!("解压更新包失败: {}", err)),
                    }
                } else {
                    UpdateStatus::ReadyToRestart {
                        new_exe_path: dest,
                        is_installer,
                    }
                }
            })();

            let _ = progress_tx.send(Some(status));
        })
        .detach();

    // 前台主线程消费进度事件并触发 UI 刷新
    cx.foreground_executor()
        .spawn(async move {
            let mut final_status = None;
            loop {
                while let Ok(msg) = progress_rx.try_recv() {
                    if let Some(status) = msg {
                        match &status {
                            UpdateStatus::Downloading(_) => {
                                let _ = async_cx.update(|cx| {
                                    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
                                        bridge.status = status;
                                    });
                                    cx.refresh_windows();
                                });
                            }
                            _ => {
                                final_status = Some(status);
                                break;
                            }
                        }
                    }
                }

                if let Some(status) = final_status {
                    let _ = async_cx.update(|cx| {
                        cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
                            bridge.status = status;
                        });
                        cx.refresh_windows();
                    });
                    break;
                }

                async_cx
                    .background_executor()
                    .timer(std::time::Duration::from_millis(50))
                    .await;
            }
        })
        .detach();
}

/// 解压 Zip 更新包并返回可执行文件路径
fn extract_zip_update(zip_path: &Path, base_dir: &Path) -> Result<PathBuf, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("打开 zip 失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解析 zip 失败: {}", e))?;

    let extract_dir = base_dir.join("extracted");
    let _ = std::fs::create_dir_all(&extract_dir);

    let mut target_exe = None;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 项失败: {}", e))?;
        let outpath = match entry.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };

        if entry.is_dir() {
            let _ = std::fs::create_dir_all(&outpath);
        } else {
            if let Some(p) = outpath.parent() {
                let _ = std::fs::create_dir_all(p);
            }
            let mut outfile =
                std::fs::File::create(&outpath).map_err(|e| format!("创建解压文件失败: {}", e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("写入解压文件失败: {}", e))?;

            if outpath
                .file_name()
                .is_some_and(|name| name == "widget-rs.exe")
            {
                target_exe = Some(outpath);
            }
        }
    }

    target_exe.ok_or_else(|| "未在更新包中找到 widget-rs.exe".to_string())
}

/// 启动独立 Updater Helper 完成无缝更新并退出当前应用
pub fn apply_update_and_restart(new_exe_path: &Path, is_installer: bool, cx: &mut App) {
    if is_installer {
        // 降级模式：启动安装器向导并退出
        let _ = std::process::Command::new(new_exe_path).spawn();
        cx.quit();
        return;
    }

    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("[update] 无法获取当前程序路径: {}", e);
            return;
        }
    };

    let helper_dir = std::env::temp_dir().join("widget-rs-helper");
    let _ = std::fs::create_dir_all(&helper_dir);
    let helper_exe = helper_dir.join("widget-rs-updater.exe");

    // 将新版可执行文件复制一份作为独立 helper 进程启动
    if let Err(e) = std::fs::copy(new_exe_path, &helper_exe) {
        // 若复制失败，尝试复制当前 exe
        let _ = std::fs::copy(&current_exe, &helper_exe);
        eprintln!("[update] 准备 updater helper: {}", e);
    }

    let pid = std::process::id();
    let _ = std::process::Command::new(&helper_exe)
        .arg("--update-helper")
        .arg("--wait-pid")
        .arg(pid.to_string())
        .arg("--source")
        .arg(new_exe_path)
        .arg("--target")
        .arg(&current_exe)
        .spawn();

    cx.quit();
}
