fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "zUI");
        res.set(
            "FileDescription",
            &format!(
                "zUI {} {}",
                env!("CARGO_PKG_VERSION"),
                include_str!("release-name.txt").trim()
            ),
        );
        res.compile().unwrap();
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
    // Discover added language folders even when no Rust source has changed.
    println!("cargo:rerun-if-changed=locales");
    println!("cargo:rerun-if-changed=release-name.txt");
}
