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

        // 尝试相对 extensions 根目录读取
        let ext_candidate = std::path::Path::new("extensions").join(path);
        if ext_candidate.is_file() {
            if let Ok(bytes) = std::fs::read(ext_candidate) {
                return Ok(Some(std::borrow::Cow::Owned(bytes)));
            }
        }

        // 尝试在 extensions 各子插件目录中递归查找
        if let Ok(entries) = std::fs::read_dir("extensions") {
            for entry in entries.flatten() {
                let sub_path = entry.path().join(path);
                if sub_path.is_file() {
                    if let Ok(bytes) = std::fs::read(sub_path) {
                        return Ok(Some(std::borrow::Cow::Owned(bytes)));
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
