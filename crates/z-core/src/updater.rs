//! Обновление самого zUI из релизов GitHub.
use self_update::backends::github::{ReleaseList, Update, UpdateBuilder};
use std::time::Duration;

/// Без таймаута зависшее соединение навсегда оставляет проверку или установку «в процессе».
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

const OWNER: &str = "maybepritz";
const REPO: &str = "zUI";

fn builder(current: &str) -> UpdateBuilder {
    let mut b = Update::configure();
    b.repo_owner(OWNER)
        .repo_name(REPO)
        .bin_name("zUI")
        .target("x86_64-pc-windows-msvc")
        .current_version(current)
        .timeout(DOWNLOAD_TIMEOUT)
        .no_confirm(true)
        .show_download_progress(false);
    b
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub fn check(current: &str) -> Result<Option<String>, String> {
    let releases = ReleaseList::configure()
        .repo_owner(OWNER)
        .repo_name(REPO)
        .timeout(CHECK_TIMEOUT)
        .build()
        .map_err(err)?
        .fetch()
        .map_err(err)?;
    let Some(latest) = releases.latest() else {
        return Ok(None);
    };
    let newer = self_update::version::bump_is_greater(current, latest.version()).map_err(err)?;
    Ok(newer.then(|| latest.version().to_string()))
}

pub fn install(current: &str) -> Result<String, String> {
    let status = builder(current)
        .build()
        .map_err(err)?
        .update()
        .map_err(err)?;
    Ok(status.version().to_string())
}
