fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "zUI");
        res.set("FileDescription", "zUI");
        res.compile().unwrap();
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
