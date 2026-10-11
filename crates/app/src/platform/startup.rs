use std::path::Path;

#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    use winreg::{
        RegKey,
        enums::HKEY_CURRENT_USER
    };
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .map_err(|e| e.to_string())?;
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let runtime = z_core::paths::runtime_root();
        let command = format!("\"{}\" --autostart --runtime {}", exe.display(), quote_path(&runtime));
        key.set_value(super::APP_ID, &command).map_err(|e| e.to_string())
    } else {
        match key.delete_value(super::APP_ID) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

fn quote_path(path: &Path) -> String {
    // A quoted Windows argument ending in a backslash must double that suffix.
    let value = path.to_string_lossy();
    let trailing = value.chars().rev().take_while(|c| *c == '\\').count();
    format!("\"{}{}\"", value, "\\".repeat(trailing))
}

#[cfg(windows)]
pub fn relaunch_elevated() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        core::{
            PCWSTR,
            w
        },
        Win32::UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::SW_SHOWNORMAL
        }
    };
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let args = format!(
        "--wait-for-pid {} --runtime {}",
        std::process::id(),
        quote_path(&z_core::paths::runtime_root()),
    );
    let exe: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
    let args: Vec<u16> = args.encode_utf16().chain(Some(0)).collect();
    // ShellExecute performs the ordinary Windows UAC consent flow. Cancellation
    // leaves the current application and its processes running.
    let result = unsafe {
        ShellExecuteW(None, w!("runas"), PCWSTR(exe.as_ptr()), PCWSTR(args.as_ptr()), None, SW_SHOWNORMAL)
    };
    if result.0 as isize <= 32 {
        Err(format!("UAC: запуск отменён или недоступен ({})", result.0 as isize))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn set_autostart(_: bool) -> Result<(), String> {
    Err("Windows only".into())
}

#[cfg(not(windows))]
pub fn relaunch_elevated() -> Result<(), String> {
    Err("Windows only".into())
}

#[cfg(windows)]
pub fn system_dark() -> bool {
    use winreg::{
        RegKey,
        enums::HKEY_CURRENT_USER
    };
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|key| key.get_value::<u32, _>("AppsUseLightTheme"))
        .map(|light| light == 0).unwrap_or(false)
}

#[cfg(not(windows))]
pub fn system_dark() -> bool {
    false
}
