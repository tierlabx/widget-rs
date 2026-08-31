use gpui::*;
use serde_json::Value;
use std::io::{Read, Write};

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
    },
    /// 正在下载 (0-100%)
    Downloading(u8),
    /// 下载完成，待安装
    ReadyToInstall(std::path::PathBuf),
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
                        // 查找 *-setup.exe 资产
                        let mut download_url = String::new();
                        if let Some(assets) = body["assets"].as_array() {
                            for asset in assets {
                                if let Some(name) = asset["name"].as_str() {
                                    if name.ends_with(".exe") && name.contains("setup") {
                                        if let Some(url) = asset["browser_download_url"].as_str() {
                                            download_url = url.to_string();
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if !download_url.is_empty() {
                            UpdateStatus::Available {
                                version: latest_version.to_string(),
                                download_url,
                                release_notes,
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

/// 流式分块下载安装包并实时反馈真实百分比进度
pub fn download_update(url: String, cx: &mut App) {
    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
        bridge.status = UpdateStatus::Downloading(0);
        bridge.dismissed = false;
    });
    cx.refresh_windows();

    let (progress_tx, progress_rx) = std::sync::mpsc::channel::<Option<UpdateStatus>>();
    let async_cx = cx.to_async();

    // 后台独立执行下载与分块写入
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

                let temp_dir = std::env::temp_dir();
                let file_name = url
                    .split('/')
                    .last()
                    .unwrap_or("widget-rs-setup.exe")
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

                UpdateStatus::ReadyToInstall(dest)
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
