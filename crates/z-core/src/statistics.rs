use serde::{Deserialize, Serialize};
use std::{path::Path, time::{SystemTime, UNIX_EPOCH}};

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Statistics {
    pub uptime_seconds: u64,
    pub strategy_changes: u64,
    pub watchdog_restarts: u64,
    pub last_successful_test: Option<u64>,
    pub last_retest: Option<u64>,
}

impl Statistics {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        crate::settings::save_json(path, self)
    }
}
