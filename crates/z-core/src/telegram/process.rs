//! Owns the proxy process; readiness only checks its local TCP socket.
use super::config::{asset_name, config_endpoint, new_secret, read_config};
use anyhow::{Context, Result, bail};
use serde_json::json;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Structured observations let a UI localize state without translating log text.
#[derive(Clone, Debug)]
pub enum ProxyStatus {
    Stopped,
    Starting,
    Listening(SocketAddr),
    Exited(String),
    ProcessError(String),
    PortUnresponsive,
}

pub struct TelegramProxy {
    pub dir: PathBuf,
    child: Option<Child>,
    started: Option<Instant>,
    endpoint: Option<SocketAddr>,
    pub ready: bool,
    pub status: String,
    pub status_kind: ProxyStatus,
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
            status_kind: ProxyStatus::Stopped,
        }
    }

    pub fn installed(&self) -> bool {
        self.exe().is_file()
    }

    pub fn running(&self) -> bool {
        self.child.is_some()
    }

    pub(super) fn exe(&self) -> PathBuf {
        self.dir.join(asset_name())
    }

    /// Start the portable proxy, creating a default config only if it is absent.
    ///
    /// Rejects invalid/public bind addresses and unavailable ports. Use `poll`
    /// to observe local readiness; successful spawn alone does not imply it.
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
        self.status_kind = ProxyStatus::Starting;
        self.ready = false;
        Ok(())
    }

    /// Observe process exit and probe the configured loopback socket (at most 30ms).
    pub fn poll(&mut self) {
        let Some(child) = &mut self.child else {
            return;
        };
        match child.try_wait() {
            Ok(Some(status)) => {
                self.status_kind = ProxyStatus::Exited(status.to_string());
                self.status =
                    format!("TG WS Proxy завершился ({status}). Подробности в журнале прокси.");
                self.child = None;
                self.ready = false;
                return;
            }
            Err(e) => {
                self.status_kind = ProxyStatus::ProcessError(e.to_string());
                self.status = format!("Ошибка проверки TG WS Proxy: {e}");
                return;
            }
            Ok(None) => {}
        }
        if let Some(endpoint) = self.endpoint {
            // Loopback only, at most 30ms. This is socket readiness, not an
            // end-to-end Telegram connectivity test.
            self.ready = TcpStream::connect_timeout(&endpoint, Duration::from_millis(30)).is_ok();
            self.status_kind = if self.ready {
                ProxyStatus::Listening(endpoint)
            } else if self
                .started
                .is_some_and(|t| t.elapsed() > Duration::from_secs(30))
            {
                ProxyStatus::PortUnresponsive
            } else {
                ProxyStatus::Starting
            };
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

    /// Stop the owned process tree and wait for exit, retaining it on failure.
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
        self.status_kind = ProxyStatus::Stopped;
        Ok(())
    }
}

impl Drop for TelegramProxy {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
