use std::path::Path;

/// 从 URL 字符串中提取干净的 host/domain
pub fn extract_domain(url_str: &str) -> Option<String> {
    let without_proto = url_str
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    let host = without_proto
        .split('/')
        .next()?
        .split('?')
        .next()?
        .split('#')
        .next()?
        .trim();

    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

/// 智能优化 Favicon 像素对比度：
/// 若检测到非透明像素大部分属于深暗色（如 GitHub 纯黑章鱼猫），自动将其提亮为明亮白色，避免在深色磨砂背景下看不清
fn optimize_dark_favicon(img: image::DynamicImage) -> image::DynamicImage {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    if width == 0 || height == 0 {
        return img;
    }

    let mut visible_pixels = 0usize;
    let mut total_luminance = 0u64;

    for pixel in rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        if a > 40 {
            // 可见像素加权亮度: (R*299 + G*587 + B*114) / 1000
            let lum = (r as u64 * 299 + g as u64 * 587 + b as u64 * 114) / 1000;
            total_luminance += lum;
            visible_pixels += 1;
        }
    }

    if visible_pixels > 0 {
        let avg_lum = total_luminance / visible_pixels as u64;
        // 若平均亮度 < 85（纯黑或深灰暗色图标，典型如 GitHub 纯黑 logo）
        if avg_lum < 85 {
            let mut bright_rgba = rgba.clone();
            for pixel in bright_rgba.pixels_mut() {
                let [r, g, b, a] = pixel.0;
                if a > 40 {
                    let lum = (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000;
                    if lum < 100 {
                        // 将过暗像素自适应平滑映射至明亮白灰色 (#f1f5f9)
                        let new_val = (255 - lum.min(255) as u8).max(230);
                        *pixel = image::Rgba([new_val, new_val, new_val, a]);
                    }
                }
            }
            return image::DynamicImage::ImageRgba8(bright_rgba);
        }
    }

    img
}

use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

static FETCHING_DOMAINS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// 快速查询本地已有的 Favicon 缓存路径（纯本地文件检查，绝不发起网络请求，主线程零阻塞）
pub fn get_cached_favicon(url_str: &str, cache_dir: &Path) -> Option<String> {
    let domain = extract_domain(url_str)?;
    let file_hash = crate::system::icon::hash_path(&domain);
    let cached_png = cache_dir.join(format!("web_{file_hash}.png"));

    if cached_png.exists() {
        if let Ok(meta) = std::fs::metadata(&cached_png) {
            if meta.len() > 0 {
                return Some(cached_png.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// 在后台线程中静默抓取指定 URL 的 Favicon 并持久化（带域名去重与超时防抖）
pub fn fetch_and_cache_favicon(url_str: &str, cache_dir: &Path) -> Option<String> {
    let domain = extract_domain(url_str)?;

    // 1. 若本地已有有效缓存，直接返回
    if let Some(cached) = get_cached_favicon(url_str, cache_dir) {
        return Some(cached);
    }

    // 2. 避免并发重复请求同一个域名
    {
        let mut fetching = FETCHING_DOMAINS.lock().ok()?;
        if fetching.contains(&domain) {
            return None;
        }
        fetching.insert(domain.clone());
    }

    // 确保退出时释放正在抓取的标记
    struct DomainGuard(String);
    impl Drop for DomainGuard {
        fn drop(&mut self) {
            if let Ok(mut fetching) = FETCHING_DOMAINS.lock() {
                fetching.remove(&self.0);
            }
        }
    }
    let _guard = DomainGuard(domain.clone());

    let file_hash = crate::system::icon::hash_path(&domain);
    let cached_png = cache_dir.join(format!("web_{file_hash}.png"));

    // 3. 候选 Favicon 地址列表（包含原生站内 favicon、全球 CDN 与国内高可用镜像）
    let candidate_urls = [
        format!("https://{domain}/favicon.ico"),
        format!("https://icon.horse/icon/{domain}"),
        format!("http://{domain}/favicon.ico"),
        format!("https://api.iowen.cn/favicon/{domain}.png"),
        format!("https://www.google.com/s2/favicons?domain={domain}&sz=64"),
    ];

    for candidate in candidate_urls {
        if let Ok(bytes) = download_favicon_bytes(&candidate) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                // 执行暗色图标对比度自适应提亮优化
                let processed_img = optimize_dark_favicon(img);

                // 统一转存为标准 PNG 缓存
                if processed_img
                    .save_with_format(&cached_png, image::ImageFormat::Png)
                    .is_ok()
                {
                    return Some(cached_png.to_string_lossy().to_string());
                }
            }
        }
    }

    // 4. 若所有网络源均无 Favicon（例如 5312.com 等未提供图标站点），智能生成品牌首字母彩色专属图标保底
    if crate::system::letter_icon::generate_fallback_letter_icon(&domain, &cached_png) {
        return Some(cached_png.to_string_lossy().to_string());
    }

    None
}

fn parse_url(url: &str) -> Option<(bool, String, u16, String)> {
    let (is_https, rest) = if let Some(stripped) = url.strip_prefix("https://") {
        (true, stripped)
    } else if let Some(stripped) = url.strip_prefix("http://") {
        (false, stripped)
    } else {
        return None;
    };

    let mut parts = rest.splitn(2, '/');
    let host_port = parts.next()?;
    let path = format!("/{}", parts.next().unwrap_or(""));

    let mut hp_parts = host_port.splitn(2, ':');
    let host = hp_parts.next()?.to_string();
    let port = if let Some(p) = hp_parts.next() {
        p.parse().ok()?
    } else if is_https {
        443
    } else {
        80
    };

    Some((is_https, host, port, path))
}

/// 单次下载 Favicon 二进制数据（使用 Windows 原生 WinHttp，零额外内存开销，自带系统证书）
fn download_favicon_bytes(url: &str) -> Result<Vec<u8>, ()> {
    let (is_https, host, port, path) = parse_url(url).ok_or(())?;

    #[cfg(windows)]
    unsafe {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Networking::WinHttp::*;

        fn to_wide(s: &str) -> Vec<u16> {
            OsStr::new(s).encode_wide().chain(Some(0)).collect()
        }

        let user_agent = to_wide("Mozilla/5.0 (Windows NT 10.0; Win64; x64)");
        let session = WinHttpOpen(
            user_agent.as_ptr(),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            std::ptr::null(),
            std::ptr::null(),
            0,
        );
        if session.is_null() {
            return Err(());
        }

        // 超时设置（解析1s，连接1.5s，发送1.5s，接收1.5s）
        WinHttpSetTimeouts(session, 1000, 1500, 1500, 1500);

        let wide_host = to_wide(&host);
        let connect = WinHttpConnect(session, wide_host.as_ptr(), port, 0);
        if connect.is_null() {
            WinHttpCloseHandle(session);
            return Err(());
        }

        let wide_path = to_wide(&path);
        let flags = if is_https { WINHTTP_FLAG_SECURE } else { 0 };
        let request = WinHttpOpenRequest(
            connect,
            to_wide("GET").as_ptr(),
            wide_path.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            flags,
        );
        if request.is_null() {
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return Err(());
        }

        let send_ok =
            WinHttpSendRequest(request, std::ptr::null(), 0, std::ptr::null_mut(), 0, 0, 0);
        if send_ok == 0 || WinHttpReceiveResponse(request, std::ptr::null_mut()) == 0 {
            WinHttpCloseHandle(request);
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return Err(());
        }

        let mut status_code: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        let query_ok = WinHttpQueryHeaders(
            request,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            std::ptr::null(),
            &mut status_code as *mut _ as *mut _,
            &mut size,
            std::ptr::null_mut(),
        );

        if query_ok == 0 || !(200..300).contains(&status_code) {
            WinHttpCloseHandle(request);
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return Err(());
        }

        let mut bytes = Vec::new();
        let mut buffer = [0u8; 8192];
        loop {
            let mut bytes_read = 0u32;
            let read_ok = WinHttpReadData(
                request,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
            );
            if read_ok == 0 || bytes_read == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..bytes_read as usize]);
            if bytes.len() > 1024 * 1024 {
                break;
            }
        }

        WinHttpCloseHandle(request);
        WinHttpCloseHandle(connect);
        WinHttpCloseHandle(session);

        if bytes.len() < 8 || bytes.len() > 1024 * 1024 {
            return Err(());
        }

        let head = &bytes[..bytes.len().min(64)];
        if head.starts_with(b"<!DOCTYPE")
            || head.starts_with(b"<html")
            || head.starts_with(b"<HTML")
        {
            return Err(());
        }

        Ok(bytes)
    }

    #[cfg(not(windows))]
    {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_github_favicon() {
        let temp_dir = std::env::temp_dir();
        let res = fetch_and_cache_favicon("https://github.com", &temp_dir);
        println!("Fetched github favicon: {:?}", res);
        assert!(res.is_some());
    }

    #[test]
    fn test_fetch_fallback_favicon() {
        let temp_dir = std::env::temp_dir();
        let res = fetch_and_cache_favicon("https://5312.com", &temp_dir);
        println!("Fetched 5312.com fallback favicon: {:?}", res);
        assert!(res.is_some());
    }
}
