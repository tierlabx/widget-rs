/// Widget RS 应用的构建脚本
///
/// 在 Windows 平台上，此脚本会编译 `assets/logos/icon.ico` 为 Windows 资源，
/// 以便生成的可执行文件拥有正确的应用程序图标。
fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        // 检查图标文件是否存在，如果存在则将其设置为应用程序图标
        if std::path::Path::new("../../assets/logos/icon.ico").exists() {
            res.set_icon("../../assets/logos/icon.ico");
            if let Err(e) = res.compile() {
                println!("cargo:warning=Failed to compile Windows resources: {}", e);
            }
        }
    }
}
