use super::asset_name;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path, time::Duration};

const API_URL: &str = "https://api.github.com/repos/Flowseal/tg-ws-proxy/releases/latest";
const MAX_DOWNLOAD: u64 = 150 * 1024 * 1024;

/// Called from a worker thread. An existing executable is never overwritten.
pub fn install(dir: &Path) -> Result<String> {
    if dir.join(asset_name()).exists() {
        bail!("TG WS Proxy уже установлен");
    }
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let client = reqwest::Client::builder()
            .user_agent("zapret-ui/0.1")
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(180))
            .build()?;
        let metadata = client
            .get(API_URL)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let release: Value = serde_json::from_str(&metadata)?;
        let asset = release["assets"]
            .as_array()
            .context("Нет списка файлов релиза")?
            .iter()
            .find(|a| a["name"].as_str() == Some(asset_name()))
            .context("В релизе нет сборки Windows для этой архитектуры")?;
        let url = asset["browser_download_url"]
            .as_str()
            .context("Нет ссылки на загрузку")?;
        if !url.starts_with("https://github.com/Flowseal/tg-ws-proxy/releases/download/") {
            bail!("Неожиданный адрес загрузки");
        }
        let digest = asset["digest"]
            .as_str()
            .and_then(|s| s.strip_prefix("sha256:"))
            .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .context(
                "GitHub не предоставил SHA-256. \
                Используйте страницу официального релиза для ручной установки.",
            )?;
        let size = asset["size"]
            .as_u64()
            .filter(|s| *s > 0 && *s <= MAX_DOWNLOAD)
            .context("Недопустимый размер файла")?;
        fs::create_dir_all(dir)?;
        let part = dir.join(format!("{}.part", asset_name()));
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&part)
            .context(
                "Не удалось создать файл загрузки. \
                Возможно, установка уже запущена или остался файл .part",
            )?;
        let result: Result<()> = async {
            let mut response = client.get(url).send().await?.error_for_status()?;
            let mut hash = Sha256::new();
            let mut received = 0u64;
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
            }
            crate::download::verify(size, received, digest, &format!("{:x}", hash.finalize()))?;
            if header != b"MZ" {
                bail!("Загруженный файл не является Windows EXE");
            }
            output.sync_all()?;
            fs::hard_link(&part, dir.join(asset_name()))?;
            Ok(())
        }
        .await;
        drop(output);
        let _ = fs::remove_file(&part);
        result?;
        Ok(release["tag_name"]
            .as_str()
            .unwrap_or("установлена")
            .to_string())
    })
}

