fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        if std::path::Path::new("logos/icon.ico").exists() {
            res.set_icon("logos/icon.ico");
            if let Err(e) = res.compile() {
                println!("cargo:warning=Failed to compile Windows resources: {}", e);
            }
        }
    }
}
