use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set("ProductName", "RamScope");
        res.set(
            "FileDescription",
            "RamScope - analyseur RAM / services / demarrage pour Windows",
        );
        res.set("CompanyName", "Bisse Studio");
        res.set("LegalCopyright", "Copyright (c) 2026 Bisse Studio");
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.compile().expect("failed to compile Windows resources");
    }
}
