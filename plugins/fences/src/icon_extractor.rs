use std::path::{Path, PathBuf};

/// 获取图标缓存目录
fn get_icon_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("widget_rs_fences_icons");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// 计算路径的稳定哈希字符串
fn hash_path(path: &str) -> String {
    let mut hasher: u64 = 0xcbf29ce484222325;
    for byte in path.to_lowercase().as_bytes() {
        hasher ^= *byte as u64;
        hasher = hasher.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hasher)
}

/// 查询或提取指定文件的图标缓存路径（返回 PNG 绝对路径）
pub fn get_or_extract_icon(path: &str) -> Option<String> {
    if path.trim().is_empty() {
        return None;
    }

    let cache_dir = get_icon_cache_dir();
    let file_hash = hash_path(path);
    let cached_png = cache_dir.join(format!("{file_hash}.png"));

    if cached_png.exists() {
        if let Ok(metadata) = std::fs::metadata(&cached_png) {
            if metadata.len() > 0 {
                return Some(cached_png.to_string_lossy().to_string());
            }
        }
    }

    #[cfg(windows)]
    {
        if extract_windows_icon(path, &cached_png) {
            return Some(cached_png.to_string_lossy().to_string());
        }
    }

    None
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

    let success = (|| -> Option<bool> {
        let hdc = unsafe { CreateCompatibleDC(0) };
        if hdc == 0 {
            return None;
        }

        let mut bm: BITMAP = unsafe { std::mem::zeroed() };
        let target_bm = if hbm_color != 0 { hbm_color } else { hbm_mask };
        let get_obj_res = unsafe {
            GetObjectW(
                target_bm,
                std::mem::size_of::<BITMAP>() as i32,
                &mut bm as *mut _ as *mut _,
            )
        };

        if get_obj_res == 0 {
            unsafe {
                DeleteDC(hdc);
            }
            return None;
        }

        let width = bm.bmWidth.max(1) as u32;
        let height = (if hbm_color != 0 {
            bm.bmHeight
        } else {
            bm.bmHeight / 2
        })
        .max(1) as u32;

        let mut bmi_header: BITMAPINFOHEADER = unsafe { std::mem::zeroed() };
        bmi_header.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi_header.biWidth = width as i32;
        bmi_header.biHeight = -(height as i32); // 顶部向下
        bmi_header.biPlanes = 1;
        bmi_header.biBitCount = 32;
        bmi_header.biCompression = BI_RGB;

        let pixel_count = (width * height) as usize;
        let mut color_pixels = vec![0u8; pixel_count * 4];

        if hbm_color != 0 {
            let dib_res = unsafe {
                GetDIBits(
                    hdc,
                    hbm_color,
                    0,
                    height,
                    color_pixels.as_mut_ptr() as *mut _,
                    &mut bmi_header as *mut _ as *mut _,
                    DIB_RGB_COLORS,
                )
            };
            if dib_res == 0 {
                unsafe {
                    DeleteDC(hdc);
                }
                return None;
            }
        }

        // 处理 Alpha 通道
        let mut has_alpha = false;
        if hbm_color != 0 {
            for chunk in color_pixels.chunks_exact(4) {
                if chunk[3] > 0 {
                    has_alpha = true;
                    break;
                }
            }
        }

        // 如果没有 Alpha 通道但有掩码位图，读取掩码
        let mut mask_pixels = vec![0u8; pixel_count * 4];
        if !has_alpha && hbm_mask != 0 {
            let mut mask_header = bmi_header;
            let _ = unsafe {
                GetDIBits(
                    hdc,
                    hbm_mask,
                    0,
                    height,
                    mask_pixels.as_mut_ptr() as *mut _,
                    &mut mask_header as *mut _ as *mut _,
                    DIB_RGB_COLORS,
                )
            };
        }

        unsafe {
            DeleteDC(hdc);
        }

        // BGRA 转 RGBA
        let mut rgba_buffer = vec![0u8; pixel_count * 4];
        for i in 0..pixel_count {
            let b = color_pixels[i * 4];
            let g = color_pixels[i * 4 + 1];
            let r = color_pixels[i * 4 + 2];
            let a = if has_alpha {
                color_pixels[i * 4 + 3]
            } else if hbm_mask != 0 {
                if mask_pixels[i * 4] == 0 {
                    255
                } else {
                    0
                }
            } else {
                255
            };

            rgba_buffer[i * 4] = r;
            rgba_buffer[i * 4 + 1] = g;
            rgba_buffer[i * 4 + 2] = b;
            rgba_buffer[i * 4 + 3] = a;
        }

        if let Some(img) = image::RgbaImage::from_raw(width, height, rgba_buffer) {
            let _ = img.save(output_png);
            Some(true)
        } else {
            None
        }
    })()
    .unwrap_or(false);

    unsafe {
        if hbm_color != 0 {
            DeleteObject(hbm_color);
        }
        if hbm_mask != 0 {
            DeleteObject(hbm_mask);
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
        let res = get_or_extract_icon(path);
        println!("Extract explorer.exe icon result: {:?}", res);
        assert!(res.is_some());
    }
}
