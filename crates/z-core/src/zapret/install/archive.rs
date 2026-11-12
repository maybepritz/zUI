use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use super::{MAX_ENTRIES, MAX_UNPACKED, USER_FILES};

/// Если все файлы лежат в одной папке верхнего уровня, возвращает её имя
fn single_root(archive: &mut zip::ZipArchive<fs::File>) -> Result<Option<PathBuf>> {
    let mut root: Option<std::ffi::OsString> = None;
    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i)?;
        let Some(path) = entry.enclosed_name() else {
            bail!(
                "Небезопасный путь в архиве: {}",
                String::from_utf8_lossy(entry.name_raw())
            );
        };
        let mut comps = path.components();
        let Some(first) = comps.next() else { continue };
        if comps.next().is_none() && !entry.is_dir() {
            return Ok(None); // файл прямо в корне архива
        }
        match &root {
            None => root = Some(first.as_os_str().to_owned()),
            Some(r) if r.as_os_str() != first.as_os_str() => return Ok(None),
            _ => {}
        }
    }
    Ok(root.map(PathBuf::from))
}

fn av_hint(e: io::Error, path: &Path) -> anyhow::Error {
    // 225 = ERROR_VIRUS_INFECTED
    if e.raw_os_error() == Some(225) {
        anyhow::anyhow!(
            "Антивирус заблокировал {}. Добавьте папку runtime в исключения и повторите установку",
            path.display()
        )
    } else {
        anyhow::Error::new(e).context(format!("Не удалось записать {}", path.display()))
    }
}

pub(super) fn extract_zip(zip_path: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file).context("Архив повреждён")?;
    if archive.len() > MAX_ENTRIES {
        bail!("В архиве слишком много файлов");
    }
    let strip = single_root(&mut archive)?;

    let mut total = 0u64;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let Some(path) = entry.enclosed_name() else {
            bail!(
                "Небезопасный путь в архиве: {}",
                String::from_utf8_lossy(entry.name_raw())
            );
        };
        let rel: PathBuf = match &strip {
            Some(root) => path
                .strip_prefix(root)
                .unwrap_or(path.as_path())
                .to_path_buf(),
            None => path.clone(),
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let out = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let declared = entry.size();
        if declared > MAX_UNPACKED.saturating_sub(total) {
            bail!("Распакованный размер превышает допустимый");
        }
        total += declared;
        let mut f = fs::File::create(&out).map_err(|e| av_hint(e, &out))?;
        let written =
            io::copy(&mut (&mut entry).take(declared + 1), &mut f).map_err(|e| av_hint(e, &out))?;
        if written != declared {
            bail!("Размер файла в архиве не совпал: {}", rel.display());
        }
    }
    Ok(())
}

/// Готовит файлы пользователя до изменения рабочей установки; откатывает
/// неудачное последнее переименование. Прошлая установка сохраняется для восстановления.
pub(super) fn move_into_place(staging: &Path, dir: &Path) -> Result<()> {
    if !dir.exists() {
        return fs::rename(staging, dir).context("Не удалось установить файлы");
    }
    for relative in USER_FILES {
        let source = dir.join(relative);
        if source.is_file() {
            let destination = staging.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source, &destination)
                .with_context(|| format!("Не удалось сохранить {}", source.display()))?;
        }
    }
    let backup = dir.with_file_name(format!(
        "{BACKUP_PREFIX}{}-{}",
        crate::clock::now(),
        std::process::id(),
    ));
    if backup.exists() {
        bail!("Резервная папка уже существует: {}", backup.display());
    }
    fs::rename(dir, &backup).context("Не удалось сохранить предыдущую установку")?;
    if let Err(error) = fs::rename(staging, dir) {
        fs::rename(&backup, dir).with_context(|| {
            format!(
                "Ошибка установки: {error}. Восстановите папку {} вручную",
                backup.display()
            )
        })?;
        return Err(error).context("Установка отменена; предыдущая версия восстановлена");
    }
    Ok(())
}

const BACKUP_PREFIX: &str = "zapret.bak-";

/// Оставляет только последнюю резервную копию: каждая занимает место целой установки.
/// По возможности: неудачная уборка не должна отменять уже завершённую установку.
pub(super) fn prune_backups(root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    // В имени стоит время создания в секундах, поэтому порядок имён — порядок по времени.
    let mut backups: Vec<(u64, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let created = name.strip_prefix(BACKUP_PREFIX)?.split('-').next()?;
            Some((created.parse().ok()?, entry.path()))
        })
        .collect();
    backups.sort();
    backups.pop();
    for (_, path) in backups {
        let _ = fs::remove_dir_all(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_latest_backup_is_kept() {
        let root = std::env::temp_dir().join(format!(
            "zui-backups-{}-{}",
            std::process::id(),
            crate::clock::now(),
        ));
        for name in ["zapret.bak-100-1", "zapret.bak-99-7", "zapret.bak-250-3"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        fs::create_dir_all(root.join("zapret")).unwrap();
        prune_backups(&root);
        let mut left: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        left.sort();
        assert_eq!(left, ["zapret", "zapret.bak-250-3"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_replacement_restores_previous_installation() {
        let root = std::env::temp_dir().join(format!(
            "zui-install-rollback-{}-{}",
            std::process::id(),
            crate::clock::now(),
        ));
        let installed = root.join("zapret");
        fs::create_dir_all(&installed).unwrap();
        fs::write(installed.join(".version"), "previous").unwrap();
        // Отсутствующая временная папка заставляет последнее переименование упасть.
        assert!(move_into_place(&root.join("missing"), &installed).is_err());
        assert_eq!(
            fs::read_to_string(installed.join(".version")).unwrap(),
            "previous"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
