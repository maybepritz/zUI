mod archive;

use archive::{extract_zip, move_into_place};
use std::{fs, io::Write, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const RELEASES_URL: &str = "https://github.com/Flowseal/zapret-discord-youtube/releases/latest";
const API_URL: &str =
    "https://api.github.com/repos/Flowseal/zapret-discord-youtube/releases/latest";
const DOWNLOAD_PREFIX: &str =
    "https://github.com/Flowseal/zapret-discord-youtube/releases/download/";

const MAX_DOWNLOAD: u64 = 10 * 1024 * 1024;
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

    let runtime = tokio::runtime::Runtime::new()?;
    let tag = runtime.block_on(download(&part, log))?;

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

async fn download(part: &Path, log: &dyn Fn(&str)) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("zapret-ui/0.1")
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(300))
        .build()?;

    log("Запрашиваю информацию о последнем релизе...");
    let metadata = client
        .get(API_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let release: Value = serde_json::from_str(&metadata)?;
    let tag = release["tag_name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    let assets = release["assets"]
        .as_array()
        .context("Нет списка файлов релиза")?;
    let is_zip = |a: &&Value| {
        a["name"]
            .as_str()
            .is_some_and(|n| n.to_ascii_lowercase().ends_with(".zip"))
    };
    let asset = assets
        .iter()
        .filter(is_zip)
        .find(|a| {
            a["name"]
                .as_str()
                .is_some_and(|n| n.to_ascii_lowercase().starts_with("zapret-discord-youtube"))
        })
        .or_else(|| assets.iter().find(is_zip))
        .context("В релизе нет zip-архива")?;

    let url = asset["browser_download_url"]
        .as_str()
        .context("Нет ссылки на загрузку")?;
    if !url.starts_with(DOWNLOAD_PREFIX) {
        bail!("Неожиданный адрес загрузки");
    }
    let digest = asset["digest"]
        .as_str()
        .and_then(|s| s.strip_prefix("sha256:"))
        .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .with_context(|| {
            format!("GitHub не предоставил SHA-256. Скачайте архив вручную: {RELEASES_URL}")
        })?;
    let size = asset["size"]
        .as_u64()
        .filter(|s| *s > 0 && *s <= MAX_DOWNLOAD)
        .context("Недопустимый размер файла")?;

    log(&format!(
        "Скачиваю {} ({} КБ)...",
        asset["name"].as_str().unwrap_or("архив"),
        size / 1024
    ));

    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(part)
        .context(
            "Не удалось создать файл загрузки. Возможно, установка уже идёт или остался файл zapret.zip.part",
        )?;

    let result: Result<()> = async {
        let mut response = client.get(url).send().await?.error_for_status()?;
        let mut hash = Sha256::new();
        let mut received = 0u64;
        let mut last_step = 0u64;
        let mut header: Vec<u8> = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            received += chunk.len() as u64;
            if received > size {
                bail!("Размер загрузки превышает размер релиза");
            }
            if header.len() < 2 {
                header.extend(chunk.iter().take(2 - header.len()));
            }
            hash.update(&chunk);
            output.write_all(&chunk)?;
            let step = received * 4 / size; // 25%, 50%, 75%
            if step > last_step && step < 4 {
                last_step = step;
                log(&format!("Загружено {}%", step * 25));
            }
        }
        crate::download::verify(size, received, digest, &format!("{:x}", hash.finalize()))?;
        if header != b"PK" {
            bail!("Загруженный файл не является ZIP-архивом");
        }
        output.sync_all()?;
        Ok(())
    }
    .await;

    drop(output);
    if let Err(e) = result {
        let _ = fs::remove_file(part); // файл создали мы (create_new)
        return Err(e);
    }
    Ok(tag)
}

pub fn latest_tag() -> Result<String> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let client = reqwest::Client::builder()
            .user_agent("zapret-ui/0.1")
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(30))
            .build()?;
        let body = client
            .get(API_URL)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let release: Value = serde_json::from_str(&body)?;
        release["tag_name"]
            .as_str()
            .map(str::to_string)
            .context("В ответе GitHub нет tag_name")
    })
}

pub fn update_available(dir: &Path) -> Result<Option<String>> {
    if !is_installed(dir) {
        return Ok(None);
    }
    let latest = latest_tag()?;
    let same = installed_version(dir)
        .is_some_and(|cur| cur.trim_start_matches('v') == latest.trim_start_matches('v'));
    Ok((!same).then_some(latest))
}
