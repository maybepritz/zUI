use anyhow::{Context, Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path, time::Duration};

pub struct Source {
    pub api_url: &'static str,         // .../releases/latest
    pub download_prefix: &'static str, // проверка адреса
    pub releases_url: &'static str,    // ссылка в тексте ошибки
    pub magic: [u8; 2],                // b"MZ" или b"PK"
    pub max_size: u64,
}

pub fn verify(expected_size: u64, actual_size: u64, expected: &str, actual: &str) -> Result<()> {
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("Отсутствует корректный SHA-256 официального релиза");
    }
    if actual_size != expected_size || !actual.eq_ignore_ascii_case(expected) {
        bail!("Проверка размера или SHA-256 не пройдена");
    }
    Ok(())
}

pub fn fetch_latest(
    source: &Source,
    pick: impl Fn(&[Value]) -> Option<&Value>,
    part: &Path,
    log: &dyn Fn(&str),
) -> Result<String> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let client = reqwest::Client::builder()
            .user_agent(concat!("zUI/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(300))
            .build()?;

        log("Запрашиваю информацию о последнем релизе...");

        let metadata = client
            .get(source.api_url)
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
        let asset = pick(assets).context("В релизе нет подходящего файла")?;

        let url = asset["browser_download_url"]
            .as_str()
            .context("Нет ссылки на загрузку")?;

        if !url.starts_with(source.download_prefix) {
            bail!("Неожиданный адрес загрузки");
        }

        let digest = asset["digest"]
            .as_str()
            .and_then(|s| s.strip_prefix("sha256:"))
            .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .with_context(|| {
                format!("GitHub не предоставил SHA-256. Скачайте файл вручную: {}", source.releases_url)
            })?;

        let size = asset["size"]
            .as_u64()
            .filter(|s| *s > 0 && *s <= source.max_size)
            .context("Недопустимый размер файла")?;

        log(&format!(
            "Скачиваю {} ({} КБ)...",
            asset["name"].as_str().unwrap_or("файл"),
            size / 1024
        ));

        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(part)
            .with_context(|| format!(
                "Не удалось создать {}. Возможно, установка уже идёт или файл остался от прошлой попытки",
                part.display()
            ))?;

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
                let step = received * 4 / size;
                if step > last_step && step < 4 {
                    last_step = step;
                    log(&format!("Загружено {}%", step * 25));
                }
            }
            verify(size, received, digest, &format!("{:x}", hash.finalize()))?;
            if header != source.magic {
                bail!("Загруженный файл не является ZIP-архивом");
            }
            output.sync_all()?;
            Ok(())
        }
        .await;

        drop(output);
        if let Err(e) = result {
            let _ = fs::remove_file(part);
            return Err(e);
        }
        Ok(tag)
    })
}

pub fn latest_tag(api_url: &str) -> Result<String> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let body = reqwest::Client::builder()
            .user_agent(concat!("zUI/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()?
            .get(api_url)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_tampered_truncated_or_unsigned_downloads() {
        let digest = "a".repeat(64);
        assert!(verify(10, 10, &digest, &digest.to_uppercase()).is_ok());
        assert!(verify(10, 9, &digest, &digest).is_err());
        assert!(verify(10, 10, &digest, &"b".repeat(64)).is_err());
        assert!(verify(10, 10, "", "").is_err());
    }
}
