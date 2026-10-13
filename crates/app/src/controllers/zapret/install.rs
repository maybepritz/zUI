//! Фоновые задачи установки и обновления.

use super::{ZapretController, ZapretUpdate};

impl ZapretController {
    pub fn installed(&self) -> bool {
        z_core::zapret::install::is_installed(&self.runtime_dir)
    }

    pub fn installing(&self) -> bool {
        self.job.busy()
    }

    pub fn install(&mut self) {
        if self.installing() || self.locked || self.is_running() {
            return;
        }
        self.run_install(false);
    }

    pub fn check_update(&mut self) {
        if self.check_job.busy() || self.installing() || !self.installed() {
            return;
        }
        let dir = self.runtime_dir.clone();
        self.check_job.spawn(move || {
            z_core::zapret::install::update_available(&dir).map_err(|e| format!("{e:#}"))
        });
    }

    pub fn update(&mut self) {
        if !matches!(self.update_state, ZapretUpdate::Available(_))
            || self.installing()
            || self.locked
        {
            return;
        }
        self.restart_after = self.is_running();
        // winws.exe и WinDivert64.sys заняты, пока процесс жив
        self.stop();
        if self.zapret.pid.is_some() {
            self.restart_after = false;
            return;
        }
        self.update_state = ZapretUpdate::Updating;
        self.run_install(true);
    }

    pub fn dismiss_update(&mut self) {
        if matches!(self.update_state, ZapretUpdate::Available(_)) {
            self.update_state = ZapretUpdate::Idle;
        }
    }

    pub(super) fn run_install(&mut self, upgrade: bool) {
        if self.installing() {
            return;
        }
        let log = self.log.clone();
        let dir = self.runtime_dir.clone();
        self.job.spawn(move || {
            let push = |m: &str| log.add(format!("[install] {m}"));
            let result = if upgrade {
                z_core::zapret::install::update(&dir, &push)
            } else {
                z_core::zapret::install::install(&dir, &push)
            };
            result.map_err(|e| format!("{e:#}"))
        });
    }
}
