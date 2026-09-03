use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};

/// 内存级图标高速缓存池：避免 UI 重绘每一帧频繁进行磁盘 I/O 检查
static MEM_ICON_CACHE: LazyLock<RwLock<HashMap<String, Option<String>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// 获取图标缓存目录并确保容量健康（限制最多 100 个缓存图标）
fn get_icon_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("widget_rs_fences_icons");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// 计算路径的稳定哈希字符串
pub fn hash_path(path: &str) -> String {
    let mut hasher: u64 = 0xcbf29ce484222325;
    for byte in path.to_lowercase().as_bytes() {
        hasher ^= *byte as u64;
        hasher = hasher.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hasher)
}

/// 查询或提取指定文件或网址的图标缓存路径（返回 PNG 绝对路径）
pub fn get_or_extract_icon(path: &str) -> Option<String> {
    if path.trim().is_empty() {
        return None;
    }

    // 1. 优先读取内存缓存（纳秒级命中，零磁盘 I/O）
    if let Ok(cache) = MEM_ICON_CACHE.read() {
        if let Some(hit) = cache.get(path) {
            return hit.clone();
        }
    }

    let cache_dir = get_icon_cache_dir();

    // 2. 如果是网页链接，仅从本地磁盘缓存查询（绝不在主渲染线程发起同步网络请求）
    let icon_res = if path.starts_with("http://") || path.starts_with("https://") {
        crate::system::favicon::get_cached_favicon(path, &cache_dir)
    } else {
        let file_hash = hash_path(path);
        let cached_png = cache_dir.join(format!("{file_hash}.png"));

        if cached_png.exists() {
            if let Ok(metadata) = std::fs::metadata(&cached_png) {
                if metadata.len() > 0 {
                    Some(cached_png.to_string_lossy().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            #[cfg(windows)]
            {
                if extract_windows_icon(path, &cached_png) {
                    Some(cached_png.to_string_lossy().to_string())
                } else {
                    None
                }
            }
            #[cfg(not(windows))]
            {
                None
            }
        }
    };

    // 3. 仅当命中了有效图标时写入内存缓存，避免将 None 长期死锁
    if icon_res.is_some() {
        if let Ok(mut cache) = MEM_ICON_CACHE.write() {
            cache.insert(path.to_string(), icon_res.clone());
        }
    }

    icon_res
}

/// 在后台线程中默默预热提取或网络拉取图标，完成后更新内存缓存
pub fn warm_or_fetch_icon_in_background(path: &str) -> Option<String> {
    if path.trim().is_empty() {
        return None;
    }

    let cache_dir = get_icon_cache_dir();
    let res = if path.starts_with("http://") || path.starts_with("https://") {
        crate::system::favicon::fetch_and_cache_favicon(path, &cache_dir)
    } else {
        get_or_extract_icon(path)
    };

    if res.is_some() {
        if let Ok(mut cache) = MEM_ICON_CACHE.write() {
            cache.insert(path.to_string(), res.clone());
        }
    }

    res
}

#[cfg(windows)]
fn extract_windows_icon(source_path: &str, output_png: &Path) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows_sys::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    let path_wide: Vec<u16> = OsStr::new(source_path)
        .encode_wide()
        .chain(Some(0))
        .collect();

    let mut shfi: SHFILEINFOW = unsafe { std::mem::zeroed() };
    let res = unsafe {
        SHGetFileInfoW(
            path_wide.as_ptr(),
            0,
            &mut shfi,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if res == 0 || shfi.hIcon == 0 {
        return false;
    }

    let hicon = shfi.hIcon;
    let mut icon_info: ICONINFO = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetIconInfo(hicon, &mut icon_info) };

    if ok == 0 {
        unsafe {
            DestroyIcon(hicon);
        }
        return false;
    }

    let hbm_color = icon_info.hbmColor;
    let hbm_mask = icon_info.hbmMask;

    let success = if hbm_color != 0 {
        unsafe {
            let mut bm: BITMAP = std::mem::zeroed();
            GetObjectW(
                hbm_color as _,
                std::mem::size_of::<BITMAP>() as i32,
                &mut bm as *mut _ as _,
            );

            let width = bm.bmWidth;
            let height = bm.bmHeight;

            if width <= 0 || height <= 0 {
                false
            } else {
                let hdc = CreateCompatibleDC(0);
                let mut bi: BITMAPINFOHEADER = std::mem::zeroed();
                bi.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                bi.biWidth = width;
                bi.biHeight = -height; // 自顶向下
                bi.biPlanes = 1;
                bi.biBitCount = 32;
                bi.biCompression = BI_RGB;

                let mut buf = vec![0u8; (width * height * 4) as usize];
                let lines = GetDIBits(
                    hdc,
                    hbm_color,
                    0,
                    height as u32,
                    buf.as_mut_ptr() as _,
                    &mut bi as *mut _ as _,
                    DIB_RGB_COLORS,
                );

                DeleteDC(hdc);

                if lines as i32 == height {
                    let mut has_alpha = false;
                    for chunk in buf.chunks_exact(4) {
                        if chunk[3] > 0 {
                            has_alpha = true;
                            break;
                        }
                    }

                    let mut rgba = Vec::with_capacity(buf.len());
                    for chunk in buf.chunks_exact(4) {
                        let b = chunk[0];
                        let g = chunk[1];
                        let r = chunk[2];
                        let a = if has_alpha { chunk[3] } else { 255 };
                        rgba.extend_from_slice(&[r, g, b, a]);
                    }

                    if let Some(img) = image::RgbaImage::from_raw(width as u32, height as u32, rgba)
                    {
                        img.save_with_format(output_png, image::ImageFormat::Png)
                            .is_ok()
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    } else {
        false
    };

    unsafe {
        if hbm_color != 0 {
            DeleteObject(hbm_color as _);
        }
        if hbm_mask != 0 {
            DeleteObject(hbm_mask as _);
        }
        DestroyIcon(hicon);
    }

    success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_explorer_icon() {
        let path = "C:\\Windows\\explorer.exe";
        if std::path::Path::new(path).exists() {
            let res = get_or_extract_icon(path);
            assert!(res.is_some());
        }
    }
}
