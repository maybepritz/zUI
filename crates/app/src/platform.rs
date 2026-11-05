pub const APP_ID: &str = "zUI.App";
mod handoff;
mod autostart;
pub use handoff::wait_for_previous;
pub use autostart::set_autostart;
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

/// Перезапускает zUI с правами администратора через запрос UAC.
#[cfg(windows)]
pub fn relaunch_elevated() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
        core::{PCWSTR, w},
    };
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let args = format!(
        "--wait-for-pid {} --runtime {}",
        std::process::id(),
        autostart::quote_path(&z_core::paths::runtime_root()),
    );
    let exe: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
    let args: Vec<u16> = args.encode_utf16().chain(Some(0)).collect();
    // ShellExecute показывает обычный запрос UAC Windows. Отказ
    // оставляет текущее приложение и его процессы работать.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("runas"),
            PCWSTR(exe.as_ptr()),
            PCWSTR(args.as_ptr()),
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        Err(crate::i18n::tr_args(
            "uac-launch-failed",
            &[("code", (result.0 as isize).into())],
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn relaunch_elevated() -> Result<(), String> {
    Err("Windows only".into())
}

/// Включена ли тёмная тема Windows.
#[cfg(windows)]
pub fn system_dark() -> bool {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|key| key.get_value::<u32, _>("AppsUseLightTheme"))
        .map(|light| light == 0)
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn system_dark() -> bool {
    false
}
