//! Уборка перед удалением zUI: `zUI.exe --cleanup`, затем можно удалить папку.
//!
//! Без неё на компьютере остались бы DNS стороннего сервиса, который включил zUI,
//! и задача автозапуска, указывающая на удалённый файл.
use super::autostart;
use z_core::dns::DnsManager;

/// Запрошена ли уборка вместо обычного запуска.
pub fn requested() -> bool {
    std::env::args_os().any(|arg| arg == "--cleanup")
}

/// Возвращает DNS и убирает автозапуск. Окон не показывает; если нужны права
/// администратора, Windows спросит их один раз через UAC.
pub fn run() {
    let root = z_core::paths::runtime_root();
    let mut dns = DnsManager::load(root.join("dns_state.json"));
    let needs_admin = dns.is_active() || autostart::elevated_autostart();
    if needs_admin && !z_core::dns::is_elevated() {
        // Запись в `Run` убирается и без прав: она останется убранной при отказе в UAC.
        let _ = autostart::set_autostart(false);
        elevated::run_and_wait(&format!(
            "--cleanup --runtime {}",
            autostart::quote_path(&root)
        ));
        return;
    }
    let _ = autostart::set_autostart(false);
    if dns.is_active() {
        let _ = dns.restore();
    }
}

#[cfg(windows)]
mod elevated {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::{
            Foundation::CloseHandle,
            System::Threading::WaitForSingleObject,
            UI::{
                Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW},
                WindowsAndMessaging::SW_HIDE,
            },
        },
        core::{PCWSTR, w},
    };

    /// Ждём повышенную копию до конца её работы, чтобы папку не удалили раньше времени.
    const WAIT_MS: u32 = 120_000;

    pub(super) fn run_and_wait(arguments: &str) {
        let Ok(exe) = std::env::current_exe() else {
            return;
        };
        let exe: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
        let arguments: Vec<u16> = arguments.encode_utf16().chain(Some(0)).collect();
        let mut info = SHELLEXECUTEINFOW {
            cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            lpVerb: w!("runas"),
            lpFile: PCWSTR(exe.as_ptr()),
            lpParameters: PCWSTR(arguments.as_ptr()),
            nShow: SW_HIDE.0,
            ..Default::default()
        };
        // SAFETY: строки живут до конца вызова; дескриптор процесса закрывается здесь же.
        unsafe {
            // Ошибка означает отказ в UAC: настройки остаются как есть.
            if ShellExecuteExW(&mut info).is_ok() && !info.hProcess.is_invalid() {
                WaitForSingleObject(info.hProcess, WAIT_MS);
                let _ = CloseHandle(info.hProcess);
            }
        }
    }
}

#[cfg(not(windows))]
mod elevated {
    pub(super) fn run_and_wait(_: &str) {}
}
