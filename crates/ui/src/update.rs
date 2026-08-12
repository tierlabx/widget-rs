use gpui::*;
use serde_json::Value;

/// 更新检查状态
#[derive(Clone)]
pub enum UpdateStatus {
    /// 初始状态
    Idle,
    /// 正在检查中
    Checking,
    /// 发现新版本
    Available {
        version: String,
        download_url: String,
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
}

impl Global for MainWindowUpdateBridge {}

pub fn check_for_update(cx: &mut App) {
    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
        bridge.status = UpdateStatus::Checking;
    });
    cx.refresh_windows();

    let mut async_cx = cx.to_async();
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

pub fn download_update(url: String, cx: &mut App) {
    cx.update_global::<MainWindowUpdateBridge, _>(|bridge, _| {
        // Since we don't track progress, just show Downloading(0) as a loading state
        bridge.status = UpdateStatus::Downloading(0);
    });
    cx.refresh_windows();

    let mut async_cx = cx.to_async();
    cx.foreground_executor()
        .spawn(async move {
            let status = async_cx
                .background_executor()
                .spawn(async move {
                    let resp = match ureq::get(&url)
                        .header("User-Agent", "widget-rs-updater")
                        .call()
                    {
                        Ok(r) => r,
                        Err(e) => return UpdateStatus::Error(format!("下载失败: {}", e)),
                    };

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

                    match std::io::copy(&mut resp.into_body().into_reader(), &mut file) {
                        Ok(_) => UpdateStatus::ReadyToInstall(dest),
                        Err(e) => UpdateStatus::Error(format!("写入文件失败: {}", e)),
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
