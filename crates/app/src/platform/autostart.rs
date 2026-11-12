//! Автозапуск zUI вместе с Windows.
//!
//! Обходу нужны права администратора, а запись в `Run` запускает программу без
//! них. Поэтому zUI, запущенный от администратора, регистрирует задачу
//! планировщика с наивысшими правами: она стартует при входе без запроса UAC.
//! Без прав создать такую задачу нельзя, и остаётся обычная запись в `Run`.
use std::path::Path;

const TASK_NAME: &str = "zUI Autostart";

#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let elevated = z_core::dns::is_elevated();
    if enabled {
        if elevated {
            windows::create_task()?;
            // Двух записей быть не должно: `Run` запустил бы вторую копию без прав.
            windows::set_run_key(false)
        } else {
            windows::set_run_key(true)
        }
    } else {
        windows::set_run_key(false)?;
        if !windows::task_exists() {
            return Ok(());
        }
        if !elevated {
            return Err(crate::i18n::tr("autostart-disable-needs-admin"));
        }
        windows::delete_task()
    }
}

/// Автозапуск включили без прав, а сейчас они есть: заменяем запись в `Run`
/// задачей с правами, чтобы после перезагрузки обход включался сам.
#[cfg(windows)]
pub fn upgrade_autostart() {
    if z_core::dns::is_elevated() && !windows::task_exists() && windows::create_task().is_ok() {
        let _ = windows::set_run_key(false);
    }
}

#[cfg(not(windows))]
pub fn upgrade_autostart() {}

/// Автозапуск зарегистрирован задачей с правами: убрать её может только администратор.
#[cfg(windows)]
pub(super) fn elevated_autostart() -> bool {
    windows::task_exists()
}

#[cfg(not(windows))]
pub(super) fn elevated_autostart() -> bool {
    false
}

pub(super) fn quote_path(path: &Path) -> String {
    // В аргументе Windows в кавычках обратную косую черту в конце нужно удвоить.
    let value = path.to_string_lossy();
    let trailing = value.chars().rev().take_while(|c| *c == '\\').count();
    format!("\"{}{}\"", value, "\\".repeat(trailing))
}

fn arguments(runtime: &Path) -> String {
    format!("--autostart --runtime {}", quote_path(runtime))
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Описание задачи планировщика. Настройки по умолчанию для zUI не годятся: задача
/// не стартует от батареи, снимается через трое суток и работает с пониженным приоритетом.
fn task_xml(exe: &Path, runtime: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>zUI autostart</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <GroupId>S-1-5-32-545</GroupId>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>4</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{}</Command>
      <Arguments>{}</Arguments>
    </Exec>
  </Actions>
</Task>
"#,
        xml_escape(&exe.to_string_lossy()),
        xml_escape(&arguments(runtime)),
    )
}

#[cfg(windows)]
mod windows {
    use super::{TASK_NAME, arguments, task_xml};
    use std::{os::windows::process::CommandExt, process::Command};
    use winreg::{
        RegKey,
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    };

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub(super) fn set_run_key(enabled: bool) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
            .map_err(|e| e.to_string())?;
        if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let command = format!(
                "\"{}\" {}",
                exe.display(),
                arguments(&z_core::paths::runtime_root())
            );
            key.set_value(crate::platform::APP_ID, &command)
                .map_err(|e| e.to_string())
        } else {
            match key.delete_value(crate::platform::APP_ID) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error.to_string()),
            }
        }
    }

    /// Планировщик хранит дерево задач в реестре, и читать его можно без прав.
    /// Непонятную ошибку считаем «есть»: лучше попросить права, чем молча оставить задачу.
    pub(super) fn task_exists() -> bool {
        let path = format!(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\{TASK_NAME}"
        );
        match RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path) {
            Ok(_) => true,
            Err(error) => error.kind() != std::io::ErrorKind::NotFound,
        }
    }

    fn schtasks(args: &[&str]) -> Result<(), String> {
        let status = Command::new("schtasks.exe")
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?
            .status;
        if status.success() {
            Ok(())
        } else {
            // Текст schtasks приходит в кодировке консоли, поэтому показываем только код.
            Err(crate::i18n::tr_args(
                "autostart-task-failed",
                &[("code", status.code().unwrap_or(-1).into())],
            ))
        }
    }

    pub(super) fn create_task() -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let xml = task_xml(&exe, &z_core::paths::runtime_root());
        // schtasks читает описание только из файла; UTF-16 с BOM понимают все версии Windows.
        let bytes: Vec<u8> = std::iter::once(0xFEFF)
            .chain(xml.encode_utf16())
            .flat_map(u16::to_le_bytes)
            .collect();
        let file = std::env::temp_dir().join(format!("zui-autostart-{}.xml", std::process::id()));
        std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
        let result = schtasks(&[
            "/Create",
            "/TN",
            TASK_NAME,
            "/XML",
            &file.to_string_lossy(),
            "/F",
        ]);
        let _ = std::fs::remove_file(&file);
        result
    }

    pub(super) fn delete_task() -> Result<(), String> {
        schtasks(&["/Delete", "/TN", TASK_NAME, "/F"])
    }
}

#[cfg(not(windows))]
pub fn set_autostart(_: bool) -> Result<(), String> {
    Err("Windows only".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn trailing_backslash_does_not_escape_the_closing_quote() {
        assert_eq!(
            quote_path(Path::new(r"C:\zUI\runtime")),
            r#""C:\zUI\runtime""#
        );
        assert_eq!(quote_path(Path::new(r"D:\")), r#""D:\\""#);
    }

    #[test]
    fn task_runs_elevated_with_the_same_runtime_folder() {
        let xml = task_xml(
            &PathBuf::from(r"C:\Users\Tom & Ann\zUI\zUI.exe"),
            &PathBuf::from(r"C:\Users\Tom & Ann\zUI\runtime"),
        );
        assert!(xml.contains("<RunLevel>HighestAvailable</RunLevel>"));
        assert!(xml.contains(r"<Command>C:\Users\Tom &amp; Ann\zUI\zUI.exe</Command>"));
        assert!(xml.contains(
            r"<Arguments>--autostart --runtime &quot;C:\Users\Tom &amp; Ann\zUI\runtime&quot;</Arguments>"
        ));
        // Настройки, без которых ноутбук от батареи остался бы без обхода.
        assert!(xml.contains("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"));
        assert!(xml.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"));
        assert!(!xml.contains(" & "));
    }
}
