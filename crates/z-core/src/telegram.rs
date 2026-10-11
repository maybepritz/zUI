use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    fs,
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

mod install;
pub use install::install;

pub const RELEASES_URL: &str = "https://github.com/Flowseal/tg-ws-proxy/releases/latest";

pub struct TelegramProxy {
    pub dir: PathBuf,
    child: Option<Child>,
    started: Option<Instant>,
    endpoint: Option<SocketAddr>,
    pub ready: bool,
    pub status: String,
}

impl TelegramProxy {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            child: None,
            started: None,
            endpoint: None,
            ready: false,
            status: "Остановлен".into(),
        }
    }

    pub fn installed(&self) -> bool {
        self.exe().is_file()
    }
    pub fn running(&self) -> bool {
        self.child.is_some()
    }
    fn exe(&self) -> PathBuf {
        self.dir.join(asset_name())
    }
    pub fn config_path(&self) -> PathBuf {
        self.dir.join("TgWsProxy_data/config.json")
    }
    pub fn log_path(&self) -> PathBuf {
        self.dir.join("TgWsProxy_data/proxy.log")
    }

    pub fn start(&mut self) -> Result<()> {
        if self.running() {
            return Ok(());
        }
        if !self.installed() {
            bail!("Сначала установите TG WS Proxy");
        }
        let path = self.config_path();
        if !path.exists() {
            crate::settings::save_json(
                &path,
                &json!({
                    "host": "127.0.0.1", "port": 1443, "secret": new_secret()?,
                    "dc_ip": ["2:149.154.167.220", "4:149.154.167.220"],
                    "check_updates": false, "autostart": false, "language": "ru"
                }),
            )?;
        }
        let config = read_config(&path)?;
        let endpoint = config_endpoint(&config)?;
        // Binding also detects another copy of the proxy or unrelated software.
        let probe = TcpListener::bind(endpoint).with_context(|| {
            format!(
                "Порт {} занят или недоступен. Закройте другой прокси либо измените config.json",
                endpoint.port()
            )
        })?;
        drop(probe);
        let mut cmd = Command::new(self.exe());
        cmd.arg("--portable")
            .current_dir(&self.dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        cmd.creation_flags(0x08000000);
        self.child = Some(cmd.spawn().context("Не удалось запустить TG WS Proxy")?);
        self.endpoint = Some(endpoint);
        self.started = Some(Instant::now());
        self.status = "Запускается…".into();
        self.ready = false;
        Ok(())
    }

    pub fn poll(&mut self) {
        let Some(child) = &mut self.child else {
            return;
        };
        match child.try_wait() {
            Ok(Some(status)) => {
                self.status =
                    format!("TG WS Proxy завершился ({status}). Подробности в журнале прокси.");
                self.child = None;
                self.ready = false;
                return;
            }
            Err(e) => {
                self.status = format!("Ошибка проверки TG WS Proxy: {e}");
                return;
            }
            Ok(None) => {}
        }
        if let Some(endpoint) = self.endpoint {
            // Loopback only, at most 30ms. This is socket readiness, not an
            // end-to-end Telegram connectivity test.
            self.ready = TcpStream::connect_timeout(&endpoint, Duration::from_millis(30)).is_ok();
            self.status = if self.ready {
                format!("Слушает {endpoint} · подключите Telegram")
            } else if self
                .started
                .is_some_and(|t| t.elapsed() > Duration::from_secs(30))
            {
                "Процесс запущен, но порт не отвечает. Откройте журнал прокси.".into()
            } else {
                "Запускается…".into()
            };
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        let Some(child) = &mut self.child else {
            return Ok(());
        };
        if child.try_wait()?.is_none() {
            #[cfg(windows)]
            {
                let output = Command::new("taskkill.exe")
                    .args(["/PID", &child.id().to_string(), "/T", "/F"])
                    .creation_flags(0x08000000)
                    .output()?;
                if !output.status.success() && child.try_wait()?.is_none() {
                    bail!(
                        "Не удалось остановить TG WS Proxy: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            #[cfg(not(windows))]
            child.kill()?;
            child.wait()?;
        }
        self.child = None;
        self.ready = false;
        self.status = "Остановлен".into();
        Ok(())
    }

    pub fn link(&self) -> Result<String> {
        proxy_link(&read_config(&self.config_path())?)
    }

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

impl Drop for TelegramProxy {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn asset_name() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "TgWsProxy_windows_arm64.exe"
    } else {
        "TgWsProxy_windows.exe"
    }
}

fn read_config(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)
        .context("Конфигурация TG WS Proxy недоступна. Сначала запустите прокси.")?;
    let config: Value = serde_json::from_str(raw.trim_start_matches('\u{feff}'))
        .context("Повреждён config.json TG WS Proxy")?;
    if !config.is_object() {
        bail!("Конфигурация TG WS Proxy должна быть JSON-объектом");
    }
    Ok(config)
}

fn config_endpoint(config: &Value) -> Result<SocketAddr> {
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

fn new_secret() -> Result<String> {
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
