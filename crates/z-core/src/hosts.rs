use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, net::IpAddr, path::{Path, PathBuf}};

const BEGIN: &str = "# BEGIN zUI managed hosts";

const END: &str = "# END zUI managed hosts";

const MAX_BYTES: usize = 256 * 1024;

pub const TEMPLATE: &str = "# zUI: вставьте сюда записи из доверенного списка.\n\
    # Формат: IP-адрес домен [другой-домен]\n\
    # Заглушка: реальных IP-адресов здесь нет.\n";

#[derive(Serialize, Deserialize)]
struct Backup {
    original: Vec<u8>,
    applied_sha256: String,
    // Also accept the pre-commit content if replacement failed or was interrupted.
    #[serde(default)]
    previous_sha256: Option<String>,
}

impl Backup {
    fn matches(&self, content: &[u8]) -> bool {
        let hash = digest(content);
        content == self.original.as_slice()
            || hash == self.applied_sha256
            || self.previous_sha256.as_ref() == Some(&hash)
    }
}

pub fn system_path() -> Result<PathBuf> {
    let root = std::env::var_os("SystemRoot").context("SystemRoot недоступен")?;
    Ok(PathBuf::from(root).join("System32/drivers/etc/hosts"))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn validate_rules(text: &str) -> Result<String> {
    if text.len() > MAX_BYTES {
        bail!("Список hosts слишком большой");
    }
    let mut domains = HashSet::new();
    let mut lines = Vec::new();
    for (index, line) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let words: Vec<_> = line.split('#').next().unwrap_or_default().split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        if words.len() < 2 {
            bail!("Строка {}: ожидаются IP и домен", index + 1);
        }
        let ip: IpAddr = words[0].parse().with_context(|| format!("Строка {}: неверный IP", index + 1))?;
        if ip.is_unspecified() || ip.is_loopback() || ip.is_multicast() {
            bail!("Строка {}: адрес не предназначен для доступа к сервису", index + 1);
        }
        let mut normalized = vec![ip.to_string()];
        for domain in &words[1..] {
            let domain = domain.to_ascii_lowercase();
            if domain.len() > 253 || !domain.contains('.') || domain.parse::<IpAddr>().is_ok()
                || domain.split('.').any(|label| label.is_empty() || label.len() > 63
                        || label.starts_with('-') || label.ends_with('-')
                        || !label.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
            {
                bail!("Строка {}: неверный домен", index + 1);
            }
            if !domains.insert(domain.clone()) {
                bail!("Домен повторяется: {domain}");
            }
            normalized.push(domain);
        }
        lines.push(normalized.join(" "));
    }
    if lines.is_empty() {
        bail!("Список пуст: замените заглушку записями из доверенного источника");
    }
    Ok(lines.join("\r\n"))
}
/// The caller supplies paths, so tests never touch the machine's real hosts file.

pub fn apply(path: &Path, backup_path: &Path, rules: &str) -> Result<()> {
    let rules = validate_rules(rules)?;
    let current = fs::read(path).context("Не удалось прочитать hosts")?;
    if current.len() > MAX_BYTES {
        bail!("Системный hosts слишком большой");
    }
    let original = if backup_path.exists() {
        let backup: Backup = serde_json::from_slice(&fs::read(backup_path)?)?;
        if !backup.matches(&current) {
            bail!("hosts изменён другой программой. Автоматическая перезапись отменена");
        }
        backup.original
    } else {
        current.clone()
    };
    let text = std::str::from_utf8(&original).context("hosts не в UTF-8: автоматическая правка отменена")?;
    if text.contains(BEGIN) || text.contains(END) {
        bail!("Найден блок zUI без корректной резервной копии");
    }
    let content = format!("{text}\r\n{BEGIN}\r\n{rules}\r\n{END}\r\n");
    let backup = Backup {
        original,
        applied_sha256: digest(content.as_bytes()),
        previous_sha256: Some(digest(&current)),
    };
    crate::settings::save_json(backup_path, &backup)?;
    // Detect edits between the initial read and commit; preserve the backup on failure.
    if fs::read(path)? != current {
        bail!("hosts изменился во время подготовки");
    }
    replace(path, content.as_bytes())
}

pub fn restore(path: &Path, backup_path: &Path) -> Result<()> {
    let backup: Backup = serde_json::from_slice(&fs::read(backup_path).context("Нет резервной копии hosts")?)?;
    let current = fs::read(path)?;
    if current != backup.original {
        if !backup.matches(&current) {
            bail!("hosts изменён другой программой. Резервная копия сохранена; автоматический откат отменён");
        }
        if fs::read(path)? != current {
            bail!("hosts изменился во время подготовки отката");
        }
        replace(path, &backup.original)?;
    }
    fs::remove_file(backup_path)?;
    Ok(())
}

fn replace(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let temp = path.with_extension("zui.tmp");
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
    let result = (|| -> Result<()> {
        fs::set_permissions(&temp, fs::metadata(path)?.permissions())?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    })();
    drop(file);
    let result = result.and_then(|_| commit(&temp, path));
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result.context("Не удалось заменить hosts. Нужны права администратора")
}

#[cfg(windows)]
fn commit(temp: &Path, path: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::Storage::FileSystem::{
            REPLACE_FILE_FLAGS,
            ReplaceFileW
        },
        core::PCWSTR,
    };
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let temp: Vec<u16> = temp.as_os_str().encode_wide().chain(Some(0)).collect();
    // Preserve the existing hosts file's ACL and attributes during replacement.
    unsafe {
        ReplaceFileW(
            PCWSTR(path.as_ptr()),
            PCWSTR(temp.as_ptr()),
            None,
            REPLACE_FILE_FLAGS(0),
            None,
            None,
        )?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn commit(temp: &Path, path: &Path) -> Result<()> {
    fs::rename(temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placeholders_and_invalid_records_never_modify_hosts() {
        for rules in [TEMPLATE, "0.0.0.0 chatgpt.com", "1.2.3.4 bad/domain", "1.2.3.4 a.com\n2.3.4.5 A.com"] {
            assert!(validate_rules(rules).is_err());
        }
        assert_eq!(validate_rules("1.2.3.4 ChatGPT.com # note").unwrap(), "1.2.3.4 chatgpt.com");
    }
    #[test]
    fn backup_restore_and_conflict_detection() {
        let dir = std::env::temp_dir().join(format!("zui-hosts-{}-{}", std::process::id(), crate::statistics::now()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hosts");
        let backup = dir.join("backup.json");
        let original = b"127.0.0.1 localhost\r\n";
        fs::write(&path, original).unwrap();
        apply(&path, &backup, "1.2.3.4 chatgpt.com").unwrap();
        // Simulate an interrupted reapply: snapshot committed, hosts unchanged.
        let mut snapshot: Backup = serde_json::from_slice(&fs::read(&backup).unwrap()).unwrap();
        snapshot.previous_sha256 = Some(snapshot.applied_sha256.clone());
        snapshot.applied_sha256 = digest(b"not yet committed");
        crate::settings::save_json(&backup, &snapshot).unwrap();
        apply(&path, &backup, "2.3.4.5 chatgpt.com").unwrap();
        let applied = fs::read(&path).unwrap();
        fs::write(&path, b"external edit").unwrap();
        assert!(restore(&path, &backup).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"external edit");
        fs::write(&path, applied).unwrap();
        restore(&path, &backup).unwrap();
        assert_eq!(fs::read(&path).unwrap(), original);
        fs::remove_dir_all(dir).unwrap();
    }
}
