//updater.rs
use self_update::backends::github::{ReleaseList, Update, UpdateBuilder};

const OWNER: &str = "maybepritz";
const REPO: &str = "zUI";

fn builder(current: &str) -> UpdateBuilder {
    let mut b = Update::configure();
    b.repo_owner(OWNER)
        .repo_name(REPO)
        .bin_name("zUI")
        .target("x86_64-pc-windows-msvc")
        .current_version(current)
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
        .build()
        .map_err(err)?
        .fetch()
        .map_err(err)?;
    let Some(latest) = releases.latest() else {
        return Ok(None);
    };
    let newer = self_update::version::bump_is_greater(current, &latest.version()).map_err(err)?;
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
