//! Portable config, validated connection links and media preferences.
use super::TelegramProxy;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

impl TelegramProxy {
    pub fn config_path(&self) -> PathBuf {
        self.dir.join("TgWsProxy_data/config.json")
    }

    pub fn log_path(&self) -> PathBuf {
        self.dir.join("TgWsProxy_data/proxy.log")
    }

    /// Read the portable config and build a validated Telegram connection link.
    pub fn link(&self) -> Result<String> {
        proxy_link(&read_config(&self.config_path())?)
    }

    /// Update DC-to-IP overrides while stopped; `clear` removes all overrides.
    pub fn fix_media(&self, clear: bool) -> Result<()> {
        if self.running() {
            bail!("Сначала остановите TG WS Proxy");
        }
        let path = self.config_path();
        let mut config = read_config(&path)?;
        config["dc_ip"] = if clear {
            json!([])
        } else {
            json!(["4:149.154.167.220"])
        };
        crate::settings::save_json(&path, &config)
    }
}

pub(super) fn asset_name() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "TgWsProxy_windows_arm64.exe"
    } else {
        "TgWsProxy_windows.exe"
    }
}

pub(super) fn read_config(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)
        .context("Конфигурация TG WS Proxy недоступна. Сначала запустите прокси.")?;
    let config: Value = serde_json::from_str(raw.trim_start_matches('\u{feff}'))
        .context("Повреждён config.json TG WS Proxy")?;
    if !config.is_object() {
        bail!("Конфигурация TG WS Proxy должна быть JSON-объектом");
    }
    Ok(config)
}

pub(super) fn config_endpoint(config: &Value) -> Result<SocketAddr> {
    let host = config["host"].as_str().unwrap_or("127.0.0.1");
    let port = config["port"]
        .as_u64()
        .context("Порт должен быть целым числом")?;
    crate::telegram_config::endpoint(host, port).map_err(anyhow::Error::msg)
}

fn proxy_link(config: &Value) -> Result<String> {
    let endpoint = config_endpoint(config)?;
    let secret = config["secret"]
        .as_str()
        .context("В конфигурации отсутствует secret")?;
    crate::telegram_config::link(endpoint, secret).map_err(anyhow::Error::msg)
}

pub(super) fn new_secret() -> Result<String> {
    #[cfg(windows)]
    {
        #[link(name = "bcrypt")]
        unsafe extern "system" {
            fn BCryptGenRandom(
                algorithm: *mut std::ffi::c_void,
                buffer: *mut u8,
                length: u32,
                flags: u32,
            ) -> i32;
        }
        let mut bytes = [0u8; 16];
        let status = unsafe { BCryptGenRandom(std::ptr::null_mut(), bytes.as_mut_ptr(), 16, 2) };
        if status < 0 {
            bail!("Не удалось создать секрет прокси: {status}");
        }
        Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
    }
    #[cfg(not(windows))]
    bail!("Интеграция TG WS Proxy предназначена для Windows")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Value {
        json!({"host":"127.0.0.1","port":1443,"secret":"0123456789abcdef0123456789abcdef"})
    }

    #[test]
    fn secure_link_matches_flowseal() {
        assert_eq!(
            proxy_link(&config()).unwrap(),
            "tg://proxy?server=127.0.0.1&port=1443&secret=dd0123456789abcdef0123456789abcdef"
        );
        let mut value = config();
        value["no_secure"] = json!(true);
        assert!(proxy_link(&value).unwrap().contains("secret=dd0123"));
    }

    #[test]
    fn reject_invalid_port_secret_and_public_bind() {
        for (key, value) in [
            ("port", json!(0)),
            ("port", json!(65536)),
            ("secret", json!("bad&port=80")),
            ("host", json!("0.0.0.0")),
        ] {
            let mut cfg = config();
            cfg[key] = value;
            assert!(proxy_link(&cfg).is_err());
        }
    }
}
