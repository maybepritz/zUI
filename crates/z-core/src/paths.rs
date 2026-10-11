use std::path::PathBuf;

pub fn runtime_root() -> PathBuf {
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--runtime" {
            if let Some(path) = args.next() {
                return PathBuf::from(path);
            }
        }
    }
    if let Some(p) = std::env::var_os("ZAPRET_UI_RUNTIME") {
        return PathBuf::from(p);
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join("runtime")))
        .unwrap_or_else(|| PathBuf::from("runtime"))
}

pub fn zapret_dir() -> PathBuf {
    runtime_root().join("zapret")
}

pub fn tg_proxy_dir() -> PathBuf {
    runtime_root().join("tg-ws-proxy")
}
