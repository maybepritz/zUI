pub const APP_ID: &str = "zUI.App";
mod handoff;
mod startup;
pub use handoff::wait_for_previous;
pub use startup::{relaunch_elevated, set_autostart, system_dark};
pub const APP_NAME: &str = "zUI";

#[cfg(windows)]
pub fn register_app_id() {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};

    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok((key, _)) = hkcu.create_subkey(format!(r"Software\Classes\AppUserModelId\{APP_ID}")) {
        let _ = key.set_value("DisplayName", &APP_NAME);
        let _ = key.set_value("IconUri", &exe.to_string_lossy().to_string());
    }
}

#[cfg(not(windows))]
pub fn register_app_id() {}
