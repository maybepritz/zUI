mod archive;

use crate::download::{self, Source};
use archive::{extract_zip, move_into_place};
use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde_json::Value;

const SOURCE: Source = Source {
    api_url: "https://api.github.com/repos/Flowseal/zapret-discord-youtube/releases/latest",
    download_prefix: "https://github.com/Flowseal/zapret-discord-youtube/releases/download/",
    releases_url: "https://github.com/Flowseal/zapret-discord-youtube/releases/latest",
    magic: *b"PK",
    max_size: 10 * 1024 * 1024,
};

const MAX_UNPACKED: u64 = 20 * 1024 * 1024;
const MAX_ENTRIES: usize = 1000;

const REQUIRED: [&str; 4] = [
    "bin/winws.exe",
    "bin/WinDivert.dll",
    "bin/WinDivert64.sys",
    "bin/cygwin1.dll",
];
const USER_FILES: [&str; 4] = [
    "lists/ipset-exclude-user.txt",
    "lists/list-general-user.txt",
    "lists/list-exclude-user.txt",
    "utils/game_filter.enabled",
];

pub fn is_installed(dir: &Path) -> bool {
    REQUIRED.iter().all(|f| dir.join(f).is_file())
}

pub fn installed_version(dir: &Path) -> Option<String> {
    fs::read_to_string(dir.join(".version"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn install(dir: &Path, log: &dyn Fn(&str)) -> Result<String> {
    install_inner(dir, log, false)
}

pub fn update(dir: &Path, log: &dyn Fn(&str)) -> Result<String> {
    install_inner(dir, log, true)
}

fn install_inner(dir: &Path, log: &dyn Fn(&str), upgrade: bool) -> Result<String> {
    if !upgrade && is_installed(dir) {
        bail!("Zapret уже установлен");
    }
    let root = dir.parent().context("Некорректный путь установки")?;
    fs::create_dir_all(root)?;

    let part = root.join("zapret.zip.part");
    let staging = root.join(".zapret-installing");

    let tag = download::fetch_latest(&SOURCE, pick_archive, &part, log)?;

    log("Распаковка...");
    if let Err(error) = fs::create_dir(&staging) {
        let _ = fs::remove_file(&part);
        return Err(error).context(
            "Не удалось создать временный каталог. Возможно, установка уже идёт или остался каталог .zapret-installing (удалите его вручную)",
        );
    }
    let result: Result<()> = (|| {
        extract_zip(&part, &staging)?;
        let missing: Vec<&str> = REQUIRED
            .iter()
            .copied()
            .filter(|f| !staging.join(f).is_file())
            .collect();
        if !missing.is_empty() {
            bail!(
                "После распаковки не хватает файлов: {}. Возможно, их удалил антивирус: добавьте {} в исключения и повторите установку",
                missing.join(", "),
                root.display()
            );
        }
        fs::write(staging.join(".version"), &tag)?;
        move_into_place(&staging, dir)
    })();

    let _ = fs::remove_file(&part);
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;
    Ok(tag)
}

fn pick_archive(assets: &[Value]) -> Option<&Value> {
    let named = |a: &Value, f: fn(&str) -> bool| {
        a["name"]
            .as_str()
            .is_some_and(|n| f(&n.to_ascii_lowercase()))
    };
    let is_zip = |a: &&Value| named(a, |n| n.ends_with(".zip"));
    assets
        .iter()
        .filter(is_zip)
        .find(|a| named(a, |n| n.starts_with("zapret-discord-youtube")))
        .or_else(|| assets.iter().find(is_zip))
}

pub fn update_available(dir: &Path) -> Result<Option<String>> {
    if !is_installed(dir) {
        return Ok(None);
    }
    let latest = download::latest_tag(SOURCE.api_url)?;
    let same = installed_version(dir)
        .is_some_and(|cur| cur.trim_start_matches('v') == latest.trim_start_matches('v'));
    Ok((!same).then_some(latest))
}
