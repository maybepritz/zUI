//! Контроллер Zapret: владелец процесса и задач установки.
mod install;
mod poll;
mod process;
mod strategies;

use super::job::Job;
use crate::i18n::Text;
use std::path::{Path, PathBuf};
use z_core::{
    log::Log,
    settings::Settings,
    zapret::{Strategy, Zapret, load_strategies},
};

#[derive(Clone, PartialEq)]
pub(crate) enum ZapretUpdate {
    Idle,
    Available(String),
    Updating,
}

pub(crate) struct ZapretController {
    zapret: Zapret,
    log: Log,
    pub runtime_dir: PathBuf,
    pub strategies: Vec<Strategy>,
    pub selected_index: usize,
    pub status: Text,
    pub update_state: ZapretUpdate,
    job: Job<Result<String, String>>,
    check_job: Job<Result<Option<String>, String>>,
    restart_after: bool,
    pub locked: bool,
    pub wanted_running: bool,
}

impl ZapretController {
    pub fn new(runtime_dir: &Path, settings: &Settings, log: Log) -> Self {
        let zapret = Zapret::new(log.clone());
        let strategies = match load_strategies(runtime_dir) {
            Ok(s) => s,
            Err(e) => {
                log.add(crate::i18n::tr_args(
                    "strategies-load-error",
                    &[("error", (format!("{e:#}")).to_string().into())],
                ));
                Vec::new()
            }
        };
        let selected_index = strategies
            .iter()
            .position(|s| s.file_name == settings.selected_strategy)
            .unwrap_or(0);
        log.add(crate::i18n::tr_args(
            "runtime-strategies",
            &[
                ("directory", (runtime_dir.display()).to_string().into()),
                ("count", (strategies.len()).into()),
            ],
        ));
        let status = if strategies.is_empty() {
            crate::i18n::text("zapret-missing-strategies")
        } else {
            crate::i18n::text("zapret-stopped")
        }
        .into();
        Self {
            zapret,
            log,
            runtime_dir: runtime_dir.to_path_buf(),
            strategies,
            selected_index,
            status,
            update_state: ZapretUpdate::Idle,
            job: Job::new(),
            check_job: Job::new(),
            restart_after: false,
            locked: false,
            wanted_running: false,
        }
    }
}
