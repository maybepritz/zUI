/// A replacement instance must not start services until the old owner has exited.
#[cfg(windows)]
pub fn wait_for_previous() -> bool {
    use windows::Win32::{
        Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0},
        System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
    };

    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg != "--wait-for-pid" {
            continue;
        }
        let Some(pid) = args.next().and_then(|arg| arg.to_str()?.parse::<u32>().ok()) else {
            return false;
        };
        if pid == std::process::id() {
            return false;
        }
        // SYNCHRONIZE grants only waiting, with no ability to terminate the process.
        let process = match unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) } {
            Ok(process) => process,
            // The parent may have already exited before this process opened it.
            Err(error) => return error.code() == ERROR_INVALID_PARAMETER.to_hresult(),
        };
        let result = unsafe { WaitForSingleObject(process, 60_000) };
        let _ = unsafe { CloseHandle(process) };
        return result == WAIT_OBJECT_0;
    }
    true
}

#[cfg(not(windows))]
pub fn wait_for_previous() -> bool {
    true
}
