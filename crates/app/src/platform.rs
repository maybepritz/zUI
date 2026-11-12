pub const APP_ID: &str = "zUI.App";
mod autostart;
pub mod cleanup;
mod handoff;
mod instance;
pub use autostart::{set_autostart, upgrade_autostart};
pub use handoff::wait_for_previous;
#[cfg(windows)]
pub use instance::activate_message;
pub use instance::{activate_running, claim_single_instance};
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

/// Дескриптор окна Windows для вызовов, которым не нужен GPUI.
#[cfg(windows)]
pub fn window_handle(window: &gpui_kit::Window) -> Option<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match HasWindowHandle::window_handle(window).ok()?.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
        _ => None,
    }
}

#[cfg(not(windows))]
pub fn window_handle(_: &gpui_kit::Window) -> Option<isize> {
    None
}

/// Красит системный заголовок окна под тему приложения: светлая полоса над
/// тёмным интерфейсом выглядит как чужое окно.
#[cfg(windows)]
pub fn set_title_bar_dark(window: Option<isize>, dark: bool) {
    use windows::Win32::{
        Foundation::HWND,
        Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute},
        UI::WindowsAndMessaging::{
            SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
        },
    };
    let Some(window) = window else {
        return;
    };
    let hwnd = HWND(window as *mut _);
    let value = i32::from(dark);
    // SAFETY: значение — BOOL своего размера; окно принадлежит этому потоку.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&raw const value).cast(),
            size_of::<i32>() as u32,
        );
        // Windows 10 перерисовывает заголовок только после смены рамки.
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

#[cfg(not(windows))]
pub fn set_title_bar_dark(_: Option<isize>, _: bool) {}

/// Убирает у окна кнопку и жест «развернуть». Содержимое zUI — колонка шириной 720:
/// на весь экран оно превращается в узкую полосу посреди пустоты.
#[cfg(windows)]
pub fn disable_maximize(window: Option<isize>) {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            GWL_STYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WS_MAXIMIZEBOX,
        },
    };
    let Some(window) = window else {
        return;
    };
    let hwnd = HWND(window as *mut _);
    // SAFETY: окно принадлежит этому потоку; меняется только один бит стиля.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongPtrW(hwnd, GWL_STYLE, style & !(WS_MAXIMIZEBOX.0 as isize));
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

#[cfg(not(windows))]
pub fn disable_maximize(_: Option<isize>) {}
