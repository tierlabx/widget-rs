#[derive(rust_embed::RustEmbed)]
#[folder = "../../assets"]
pub struct LocalAssets;

pub struct AppAssets;

impl gpui::AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if let Some(file) = LocalAssets::get(path) {
            return Ok(Some(file.data));
        }

        // 优先尝试作为磁盘文件读取
        let direct_path = std::path::Path::new(path);
        if direct_path.is_file() {
            if let Ok(bytes) = std::fs::read(direct_path) {
                return Ok(Some(std::borrow::Cow::Owned(bytes)));
            }
        }

        // 遍历所有已知的扩展存放目录（支持打包安装后的 AppData、安装包目录及本地开发目录）
        for base_dir in widget_core::get_all_extension_dirs() {
            // 1. 尝试直接相对扩展根目录查找（例如 extensions/icons/xxx.png）
            let direct_ext = base_dir.join(path);
            if direct_ext.is_file() {
                if let Ok(bytes) = std::fs::read(&direct_ext) {
                    return Ok(Some(std::borrow::Cow::Owned(bytes)));
                }
            }

            // 2. 尝试在各个插件子目录中匹配（例如 extensions/clock/icons/sunny.png）
            if let Ok(entries) = std::fs::read_dir(&base_dir) {
                for entry in entries.flatten() {
                    let sub_path = entry.path().join(path);
                    if sub_path.is_file() {
                        if let Ok(bytes) = std::fs::read(&sub_path) {
                            return Ok(Some(std::borrow::Cow::Owned(bytes)));
                        }
                    }
                }
            }
        }

        gpui_kit_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        let mut list = gpui_kit_assets::Assets.list(path).unwrap_or_default();
        for file in LocalAssets::iter() {
            if file.starts_with(path) {
                list.push(file.to_string().into());
            }
        }
        Ok(list)
    }
}
