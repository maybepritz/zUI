//! Один экземпляр zUI на папку данных: два экземпляра с одной папкой делили бы
//! settings.json и снимок DNS и ломали бы состояние друг друга.
//!
//! Экземпляры с разными папками (`--runtime`) друг другу не мешают: так сборка из
//! проекта запускается рядом с установленной копией. Обход при этом включится только
//! у одного из них — второму WinDivert вернёт ошибку запуска.

/// Короткий отпечаток папки данных для имён объектов Windows (FNV-1a).
/// Регистр не учитывается: в Windows это одна и та же папка.
fn scope() -> String {
    let path = z_core::paths::runtime_root();
    let path = std::path::absolute(&path).unwrap_or(path);
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in path.to_string_lossy().to_lowercase().bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// Имя сообщения, которым второй экземпляр просит работающий показать окно.
/// Окно подписывается на него в `tray.rs`; имя своё у каждой папки данных.
#[cfg(windows)]
pub fn activate_message() -> u32 {
    use windows::{Win32::UI::WindowsAndMessaging::RegisterWindowMessageW, core::PCWSTR};
    let name = wide(&format!("zUI.Activate.{}", scope()));
    // SAFETY: строка с завершающим нулём живёт до конца вызова.
    unsafe { RegisterWindowMessageW(PCWSTR(name.as_ptr())) }
}

/// Занимает место единственного экземпляра до выхода из процесса.
/// Возвращает `false`, если zUI с этой папкой данных уже запущен.
#[cfg(windows)]
pub fn claim_single_instance() -> bool {
    use windows::{
        Win32::{
            Foundation::{ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, GetLastError},
            System::Threading::CreateMutexW,
        },
        core::{PCWSTR, w},
    };
    let name = wide(&format!("Local\\zUI.App.Instance.{}", scope()));
    // SAFETY: имена — строки с завершающим нулём. Дескрипторы намеренно не закрываются:
    // Windows освободит мьютексы при любом завершении процесса, включая падение.
    let free = match unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) } {
        Ok(_) => unsafe { GetLastError() != ERROR_ALREADY_EXISTS },
        // Мьютекс создан экземпляром с правами администратора, а у нас их нет.
        Err(error) if error.code() == ERROR_ACCESS_DENIED.to_hresult() => false,
        // Непонятная ошибка не должна мешать запустить приложение.
        Err(_) => true,
    };
    if free {
        // Общая метка «zUI запущен» для установщика (`AppMutex` в zui.iss): он просит
        // закрыть приложение перед обновлением или удалением. На запуск она не влияет.
        let _ = unsafe { CreateMutexW(None, false, w!("Local\\zUI.App.Instance")) };
    }
    free
}

/// Просит уже работающий экземпляр показать окно (в том числе из трея).
#[cfg(windows)]
pub fn activate_running() {
    use windows::Win32::{
        Foundation::{LPARAM, WPARAM},
        UI::WindowsAndMessaging::{
            ASFW_ANY, AllowSetForegroundWindow, HWND_BROADCAST, PostMessageW,
        },
    };
    let message = activate_message();
    if message == 0 {
        return;
    }
    // SAFETY: широковещательное сообщение без указателей; его обрабатывает только zUI.
    unsafe {
        // Иначе Windows не даст работающему экземпляру выйти на передний план.
        let _ = AllowSetForegroundWindow(ASFW_ANY);
        let _ = PostMessageW(Some(HWND_BROADCAST), message, WPARAM(0), LPARAM(0));
    }
}

#[cfg(not(windows))]
pub fn claim_single_instance() -> bool {
    true
}

#[cfg(not(windows))]
pub fn activate_running() {}

#[cfg(test)]
mod tests {
    #[test]
    fn scope_is_a_stable_name_fragment() {
        let scope = super::scope();
        assert_eq!(scope.len(), 16);
        assert!(scope.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(scope, super::scope());
    }
}
