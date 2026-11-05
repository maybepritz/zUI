//! Сохранение JSON-файлов: атомарная запись, резервная копия и восстановление
//! после повреждения. Общее для настроек, статистики, DNS и прокси Telegram.
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Почему файл данных не использован как есть и что загружено вместо него.
#[derive(Debug)]
pub struct Recovery {
    pub error: anyhow::Error,
    /// Нечитаемый файл, отложенный в сторону, чтобы следующее сохранение его не затёрло.
    pub quarantined: Option<PathBuf>,
    /// Вместо значений по умолчанию загружена предыдущая читаемая копия.
    pub from_backup: bool,
}

/// Одна строка лога: ошибка, куда отложен повреждённый файл и что загружено.
impl std::fmt::Display for Recovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let moved = self.quarantined.as_deref().map(Path::display);
        let loaded = if self.from_backup {
            "backup"
        } else {
            "defaults"
        };
        match moved {
            Some(file) => write!(f, "{:#}; moved to {file}; loaded {loaded}", self.error),
            None => write!(
                f,
                "{:#}; could not move the file; loaded {loaded}",
                self.error
            ),
        }
    }
}

/// `read` возвращает `Ok(None)` для отсутствующего файла, тогда молча загружаются значения по умолчанию.
pub(crate) fn load_recovering<T: Default>(
    path: &Path,
    read: impl Fn(&Path) -> Result<Option<T>>,
) -> (T, Option<Recovery>) {
    let error = match read(path) {
        Ok(value) => return (value.unwrap_or_default(), None),
        Err(error) => error,
    };
    let moved = path.with_extension(format!("json.corrupt-{}", crate::clock::now()));
    let quarantined = fs::rename(path, &moved).is_ok().then_some(moved);
    let backup = read(&backup_path(path)).ok().flatten();
    let recovery = Recovery {
        error,
        quarantined,
        from_backup: backup.is_some(),
    };
    (backup.unwrap_or_default(), Some(recovery))
}

pub(crate) fn save_with_backup<T>(
    path: &Path,
    value: &impl Serialize,
    read: impl Fn(&Path) -> Result<Option<T>>,
) -> Result<()> {
    // Храним последнюю читаемую версию; повреждённый файл не должен заменить хорошую копию.
    // По возможности: неудачная резервная копия не должна мешать сохранить сами данные.
    if matches!(read(path), Ok(Some(_))) {
        let _ = fs::copy(path, backup_path(path));
    }
    save_json(path, value)
}

pub(crate) fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

/// Пишем рядом с файлом, затем заменяем его, чтобы прерванная запись не могла
/// обрезать последние рабочие настройки или снимок для восстановления DNS.
pub(crate) fn save_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("json.tmp");
    let mut file = fs::File::create(&temp)?;
    use std::io::Write;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp, path).with_context(|| format!("Не удалось сохранить {}", path.display()))
}
