#[derive(rust_embed::RustEmbed)]
#[folder = "../../assets"]
pub struct LocalAssets;

pub struct AppAssets;

impl gpui::AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if let Some(file) = LocalAssets::get(path) {
            return Ok(Some(file.data));
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
