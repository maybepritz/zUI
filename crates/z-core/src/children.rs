//! Дочерние процессы не должны пережить zUI: winws без владельца продолжает
//! перехватывать трафик, а следующий запуск о нём уже не знает.
//!
//! Все процессы привязываются к одному Job Object с `KILL_ON_JOB_CLOSE`. Его
//! дескриптор живёт до конца процесса, поэтому Windows завершает детей при
//! любом выходе zUI: обычном, падении или снятии через диспетчер задач.
use std::process::Child;

/// Привязывает процесс к общему заданию. Возвращает текст ошибки для журнала;
/// сам процесс при ошибке продолжает работать как раньше.
#[cfg(windows)]
pub(crate) fn adopt(child: &Child) -> Result<(), String> {
    use std::os::windows::io::AsRawHandle;
    let job = windows::job()?;
    // SAFETY: оба дескриптора действительны: задание живёт до конца процесса,
    // а дескриптор ребёнка принадлежит `child`.
    let assigned = unsafe { windows::AssignProcessToJobObject(job, child.as_raw_handle() as _) };
    if assigned == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn adopt(_: &Child) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
mod windows {
    use std::{ffi::c_void, sync::OnceLock};

    type Handle = *mut c_void;

    const EXTENDED_LIMIT_INFORMATION: i32 = 9;
    const KILL_ON_JOB_CLOSE: u32 = 0x2000;

    #[repr(C)]
    #[derive(Default)]
    struct BasicLimits {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct ExtendedLimits {
        basic: BasicLimits,
        io_counters: [u64; 6],
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(
            job: Handle,
            class: i32,
            information: *const c_void,
            length: u32,
        ) -> i32;
        pub(super) fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
    }

    /// Дескриптор задания; намеренно не закрывается до выхода из процесса.
    struct Job(Handle);
    // SAFETY: дескриптор ядра можно использовать из любого потока.
    unsafe impl Send for Job {}
    unsafe impl Sync for Job {}

    pub(super) fn job() -> Result<Handle, String> {
        static JOB: OnceLock<Result<Job, String>> = OnceLock::new();
        JOB.get_or_init(create)
            .as_ref()
            .map(|job| job.0)
            .map_err(Clone::clone)
    }

    fn create() -> Result<Job, String> {
        // SAFETY: анонимное задание без атрибутов; структура лимитов передаётся
        // со своим точным размером.
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return Err(std::io::Error::last_os_error().to_string());
            }
            let mut limits = ExtendedLimits::default();
            limits.basic.limit_flags = KILL_ON_JOB_CLOSE;
            let configured = SetInformationJobObject(
                job,
                EXTENDED_LIMIT_INFORMATION,
                (&raw const limits).cast(),
                size_of::<ExtendedLimits>() as u32,
            );
            if configured == 0 {
                let error = std::io::Error::last_os_error().to_string();
                CloseHandle(job);
                return Err(error);
            }
            Ok(Job(job))
        }
    }

    #[cfg(test)]
    #[test]
    fn limits_match_the_windows_layout() {
        // Размеры JOBOBJECT_EXTENDED_LIMIT_INFORMATION из Windows SDK.
        let expected = if cfg!(target_pointer_width = "64") {
            144
        } else {
            112
        };
        assert_eq!(size_of::<ExtendedLimits>(), expected);
    }
}

#[cfg(all(test, windows))]
mod tests {
    use std::process::{Command, Stdio};

    #[test]
    fn child_joins_the_shared_job() {
        let mut child = Command::new("cmd")
            .args(["/C", "pause"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let adopted = super::adopt(&child);
        let _ = child.kill();
        let _ = child.wait();
        adopted.unwrap();
    }
}
