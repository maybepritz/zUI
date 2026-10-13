//! Windows notification icon, owned and accessed only on the GPUI thread.
#[cfg(windows)]
mod native {
    use crate::i18n::tr;
    use gpui_kit::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::{cell::Cell, rc::Rc};
    use windows::{
        Win32::{
            Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::{Shell::*, WindowsAndMessaging::*},
        },
        core::{PCWSTR, w},
    };

    const SUBCLASS_ID: usize = 0x5a5549;
    const OPEN: usize = 1;
    const EXIT: usize = 2;

    struct State {
        data: NOTIFYICONDATAW,
        taskbar_created: u32,
        attached: Cell<bool>,
        available: Cell<bool>,
        quit: Cell<bool>,
    }

    pub struct Tray(Rc<State>);

    impl Tray {
        pub fn new(window: &Window) -> Result<Self, String> {
            let handle = HasWindowHandle::window_handle(window).map_err(|e| e.to_string())?;
            let RawWindowHandle::Win32(handle) = handle.as_raw() else {
                return Err(crate::i18n::tr("tray-window-unavailable").into());
            };
            let hwnd = HWND(handle.hwnd.get() as *mut _);
            // SAFETY: initialization runs on the owning window's UI thread.
            // LoadIconW returns a shared resource icon, which must not be destroyed.
            unsafe {
                let module = GetModuleHandleW(None).map_err(|e| e.to_string())?;
                let icon = LoadIconW(Some(HINSTANCE(module.0)), PCWSTR(1usize as *const u16))
                    .map_err(|e| e.to_string())?;
                let callback = RegisterWindowMessageW(w!("zUI.Tray.Callback"));
                let taskbar_created = RegisterWindowMessageW(w!("TaskbarCreated"));
                if callback == 0 || taskbar_created == 0 {
                    return Err(crate::i18n::tr("tray-registration-failed").into());
                }
                // Explorer normally runs without elevation, even when zUI runs as admin.
                ChangeWindowMessageFilterEx(hwnd, callback, MSGFLT_ALLOW, None)
                    .and_then(|_| {
                        ChangeWindowMessageFilterEx(hwnd, taskbar_created, MSGFLT_ALLOW, None)
                    })
                    .map_err(|e| e.to_string())?;
                let mut data = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd,
                    uID: 1,
                    uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                    uCallbackMessage: callback,
                    hIcon: icon,
                    ..Default::default()
                };
                for (slot, ch) in data.szTip.iter_mut().zip(tr("tray-tooltip").encode_utf16()) {
                    *slot = ch;
                }
                let state = Rc::new(State {
                    data,
                    taskbar_created,
                    attached: Cell::new(false),
                    available: Cell::new(false),
                    quit: Cell::new(false),
                });
                if !SetWindowSubclass(
                    hwnd,
                    Some(window_proc),
                    SUBCLASS_ID,
                    Rc::as_ptr(&state) as usize,
                )
                .as_bool()
                {
                    return Err(crate::i18n::tr("tray-handler-failed").into());
                }
                state.attached.set(true);
                let tray = Self(state);
                if !tray.0.add_icon() {
                    return Err(crate::i18n::tr("tray-icon-failed").into());
                }
                Ok(tray)
            }
        }

        pub fn hide(&self) -> bool {
            if !self.0.attached.get() || (!self.0.available.get() && !self.0.add_icon()) {
                return false;
            }
            // Queue the visibility change to avoid re-entering GPUI during its close callback.
            unsafe { ShowWindowAsync(self.0.data.hWnd, SW_HIDE).as_bool() }
        }

        pub fn take_quit_request(&self) -> bool {
            self.0.quit.replace(false)
        }

        pub fn hidden(&self) -> bool {
            unsafe {
                !IsWindowVisible(self.0.data.hWnd).as_bool() || IsIconic(self.0.data.hWnd).as_bool()
            }
        }

        /// Refresh the tooltip after a locale change; menu labels resolve on open.
        pub fn refresh_language(&self) {
            let data = self.0.localized_data();
            unsafe {
                let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
            }
        }
    }

    impl State {
        fn localized_data(&self) -> NOTIFYICONDATAW {
            let mut data = self.data;
            data.szTip.fill(0);
            for (slot, ch) in data
                .szTip
                .iter_mut()
                .take(127)
                .zip(tr("tray-tooltip").encode_utf16())
            {
                *slot = ch;
            }
            data
        }

        fn add_icon(&self) -> bool {
            let data = self.localized_data();
            let added = unsafe { Shell_NotifyIconW(NIM_ADD, &data).as_bool() };
            self.available.set(added);
            added
        }

        fn show(&self) {
            if !self.attached.get() {
                return;
            }
            // SW_SHOW preserves maximized windows; only minimized windows need restoring.
            unsafe {
                let command = if IsIconic(self.data.hWnd).as_bool() {
                    SW_RESTORE
                } else {
                    SW_SHOW
                };
                let _ = ShowWindowAsync(self.data.hWnd, command);
                let _ = SetForegroundWindow(self.data.hWnd);
            }
        }

        fn menu(&self) {
            let open: Vec<u16> = tr("tray-open").encode_utf16().chain(Some(0)).collect();
            let exit: Vec<u16> = tr("tray-exit").encode_utf16().chain(Some(0)).collect();
            // The native menu runs a nested message loop. The callback holds an Rc so
            // State remains alive even if GPUI quits and drops Tray while this is open.
            unsafe {
                let Ok(menu) = CreatePopupMenu() else {
                    self.show();
                    return;
                };
                let mut position = POINT::default();
                let result = AppendMenuW(menu, MF_STRING, OPEN, PCWSTR(open.as_ptr()))
                    .and_then(|_| AppendMenuW(menu, MF_STRING, EXIT, PCWSTR(exit.as_ptr())))
                    .and_then(|_| GetCursorPos(&mut position));
                if result.is_ok() {
                    let _ = SetForegroundWindow(self.data.hWnd);
                    let command = TrackPopupMenu(
                        menu,
                        TPM_RETURNCMD | TPM_RIGHTBUTTON,
                        position.x,
                        position.y,
                        None,
                        self.data.hWnd,
                        None,
                    )
                    .0 as usize;
                    if self.attached.get() {
                        // Required for the menu to dismiss properly on the next invocation.
                        let _ = PostMessageW(Some(self.data.hWnd), WM_NULL, WPARAM(0), LPARAM(0));
                        match command {
                            OPEN => self.show(),
                            EXIT => self.quit.set(true),
                            _ => {}
                        }
                    }
                } else {
                    self.show();
                }
                let _ = DestroyMenu(menu);
            }
        }

        fn detach(&self) {
            if self.attached.replace(false) {
                unsafe {
                    let _ = RemoveWindowSubclass(self.data.hWnd, Some(window_proc), SUBCLASS_ID);
                    let _ = Shell_NotifyIconW(NIM_DELETE, &self.data);
                }
                self.available.set(false);
            }
        }
    }

    impl Drop for Tray {
        fn drop(&mut self) {
            self.0.detach();
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        data: usize,
    ) -> LRESULT {
        // SAFETY: Tray retains this Rc while the subclass is registered; both are
        // confined to the UI thread. Keep a temporary owner across reentrant messages.
        let state = unsafe {
            let pointer = data as *const State;
            Rc::increment_strong_count(pointer);
            Rc::from_raw(pointer)
        };
        if message == state.taskbar_created {
            if !state.add_icon() {
                // Explorer may have restarted: never leave the app hidden without an icon.
                state.show();
            }
            return LRESULT(0);
        }
        if message == state.data.uCallbackMessage {
            // Legacy notify-icon protocol: wParam is the icon ID, lParam is the event.
            if wparam.0 == state.data.uID as usize {
                match lparam.0 as u32 {
                    WM_LBUTTONUP | WM_LBUTTONDBLCLK => state.show(),
                    WM_RBUTTONUP | WM_CONTEXTMENU => state.menu(),
                    _ => {}
                }
            }
            return LRESULT(0);
        }
        if message == WM_NCDESTROY {
            state.detach();
        }
        unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
    }
}

#[cfg(windows)]
pub use native::Tray;

#[cfg(not(windows))]
pub struct Tray;

#[cfg(not(windows))]
impl Tray {
    pub fn refresh_language(&self) {}
    pub fn new(_: &gpui_kit::Window) -> Result<Self, String> {
        Err(crate::i18n::tr("tray-windows-only").into())
    }

    pub fn hide(&self) -> bool {
        false
    }

    pub fn take_quit_request(&self) -> bool {
        false
    }

    pub fn hidden(&self) -> bool {
        false
    }
}
