use super::asset_name;
use crate::download::{self, Source};
use anyhow::{Result, bail};
use serde_json::Value;
use std::{fs, path::Path};

const SOURCE: Source = Source {
    api_url: "https://api.github.com/repos/Flowseal/tg-ws-proxy/releases/latest",
    download_prefix: "https://github.com/Flowseal/tg-ws-proxy/releases/download/",
    releases_url: super::RELEASES_URL,
    magic: *b"MZ",
    max_size: 150 * 1024 * 1024,
};

pub fn install(dir: &Path) -> Result<String> {
    if dir.join(asset_name()).exists() {
        bail!("TG WS Proxy уже установлен");
    }
    fs::create_dir_all(dir)?;
    let part = dir.join(format!("{}.part", asset_name()));
    // Файл от прерванной загрузки иначе блокировал бы все следующие попытки.
    let _ = fs::remove_file(&part);
    let tag = download::fetch_latest(&SOURCE, pick_exe, &part, &|_| {})?;
    let linked = fs::hard_link(&part, dir.join(asset_name()));
    let _ = fs::remove_file(&part);
    linked?;
    Ok(tag)
}

fn pick_exe(assets: &[Value]) -> Option<&Value> {
    assets
        .iter()
        .find(|a| a["name"].as_str() == Some(asset_name()))
}
