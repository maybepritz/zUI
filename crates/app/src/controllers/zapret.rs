use super::job::{Job, JobPoll};
use gpui_kit::SharedString;
use std::path::{Path, PathBuf};
use z_core::{
    log::Log,
    settings::Settings,
    zapret::{Strategy, Zapret, load_strategies},
};

#[derive(Clone, PartialEq)]
pub enum ZapretUpdate {
    Idle,
    Available(String),
    Updating,
}

pub struct ZapretController {
    zapret: Zapret,
    log: Log,
    pub runtime_dir: PathBuf,
    pub strategies: Vec<Strategy>,
    pub selected_index: usize,
    pub status: SharedString,
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
                log.add(format!("Ошибка загрузки стратегий: {e:#}"));
                Vec::new()
            }
        };
        let selected_index = strategies
            .iter()
            .position(|s| s.file_name == settings.selected_strategy)
            .unwrap_or(0);
        log.add(format!(
            "Каталог runtime: {} · стратегий: {}",
            runtime_dir.display(),
            strategies.len()
        ));
        let status = if strategies.is_empty() {
            "Нет стратегий. Проверьте runtime/zapret."
        } else {
            "Остановлено"
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

    pub fn lock(&mut self) -> bool {
        self.stop();
        if self.zapret.pid.is_some() {
            return false;
        }
        self.locked = true;
        true
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn selected(&self) -> Option<&Strategy> {
        self.strategies.get(self.selected_index)
    }

    pub fn select(&mut self, index: usize) -> Option<String> {
        if self.locked || self.installing() || index >= self.strategies.len() {
            return None;
        }
        let running = self.is_running();
        let previous = self.selected_index;
        self.selected_index = index;
        if running {
            self.stop();
            if !self.start() {
                self.selected_index = previous;
                self.start();
                return None;
            }
        }
        self.log.add(format!(
            "Выбрана стратегия: {}",
            self.strategies[index].name
        ));
        Some(self.strategies[index].file_name.clone())
    }

    pub fn is_running(&mut self) -> bool {
        let had_process = self.zapret.pid.is_some();
        let running = self.zapret.is_running();
        if had_process && !running {
            self.status = "Процесс завершился. Причина — в журнале событий.".into();
        }
        running
    }

    /// true, если процесс запущен.
    pub fn start(&mut self) -> bool {
        if self.installing() {
            self.status = "Идёт установка Zapret".into();
            return false;
        }
        if self.locked {
            self.status = "Идёт тест стратегий".into();
            return false;
        }

        let Some(strategy) = self.selected().cloned() else {
            self.status = "Ошибка: нет доступных стратегий".into();
            return false;
        };
        match self.zapret.start(
            &self.runtime_dir.join("bin"),
            &self.runtime_dir.join("lists"),
            &strategy.raw_args,
        ) {
            Ok(pid) => {
                self.wanted_running = true;
                self.status = format!("Работает (PID: {pid})").into();
                true
            }
            Err(e) => {
                let message = format!("Ошибка запуска: {e:#}");
                self.log.add(&message);
                self.status = message.into();
                false
            }
        }
    }

    pub fn stop(&mut self) {
        self.wanted_running = false;
        self.zapret.stop();
        self.status = if self.zapret.pid.is_some() {
            "Не удалось остановить процесс. См. журнал."
        } else {
            "Остановлено"
        }
        .into();
    }

    pub fn toggle(&mut self) {
        if self.is_running() {
            self.stop();
        } else {
            self.start();
        }
    }

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

    fn run_install(&mut self, upgrade: bool) {
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

    pub fn reload_strategies(&mut self) {
        let previous = self.selected().map(|s| s.file_name.clone());
        self.strategies = load_strategies(&self.runtime_dir).unwrap_or_default();
        self.selected_index = previous
            .and_then(|name| self.strategies.iter().position(|s| s.file_name == name))
            .unwrap_or(0);
        if !self.strategies.is_empty() && !self.zapret.is_running() {
            self.status = "Остановлено".into();
        }
    }

    pub fn index_of(&self, file_name: &str) -> Option<usize> {
        self.strategies
            .iter()
            .position(|s| s.file_name == file_name)
    }

    pub fn poll(&mut self) {
        self.is_running();

        match self.check_job.poll() {
            JobPoll::Done(Ok(Some(tag))) => {
                self.log.add(format!("Доступно обновление Zapret: {tag}"));
                self.update_state = ZapretUpdate::Available(tag);
            }
            JobPoll::Done(Ok(None)) => self.update_state = ZapretUpdate::Idle,
            JobPoll::Done(Err(e)) => {
                self.log.add(format!("Проверка обновления Zapret: {e}"));
            }
            JobPoll::Lost | JobPoll::Pending => {}
        }

        match self.job.poll() {
            JobPoll::Done(Ok(tag)) => {
                self.log.add(format!("Zapret {tag} установлен"));
                self.update_state = ZapretUpdate::Idle;
                self.reload_strategies();
                if std::mem::take(&mut self.restart_after) {
                    self.start();
                }
            }
            JobPoll::Done(Err(e)) => {
                self.log.add(format!("Ошибка установки: {e}"));
                self.update_state = ZapretUpdate::Idle;
                if std::mem::take(&mut self.restart_after) {
                    self.start();
                }
            }
            JobPoll::Lost => {
                self.log.add("Ошибка установки: поток прерван");
                self.update_state = ZapretUpdate::Idle;
                self.restart_after = false;
            }
            JobPoll::Pending => {}
        }
    }
}
