//! Автозапуск zUI вместе с Windows.
use std::path::Path;

#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .map_err(|e| e.to_string())?;
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let runtime = z_core::paths::runtime_root();
        let command = format!(
            "\"{}\" --autostart --runtime {}",
            exe.display(),
            quote_path(&runtime)
        );
        key.set_value(super::APP_ID, &command)
            .map_err(|e| e.to_string())
    } else {
        match key.delete_value(super::APP_ID) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub(super) fn quote_path(path: &Path) -> String {
    // В аргументе Windows в кавычках обратную косую черту в конце нужно удвоить.
    let value = path.to_string_lossy();
    let trailing = value.chars().rev().take_while(|c| *c == '\\').count();
    format!("\"{}{}\"", value, "\\".repeat(trailing))
}

#[cfg(not(windows))]
pub fn set_autostart(_: bool) -> Result<(), String> {
    Err("Windows only".into())
}
