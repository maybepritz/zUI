use super::job::{Job, JobPoll};
use z_core::log::Log;

const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, PartialEq)]
pub enum UpdateState {
    Idle,
    Checking,
    Available(String),
    Installing,
    Done(String), // установлено, нужен перезапуск
    Failed(String),
}

pub struct UpdateController {
    pub state: UpdateState,
    check_job: Job<Result<Option<String>, String>>,
    install_job: Job<Result<String, String>>,
    log: Log,
}

impl UpdateController {
    pub fn new(log: Log) -> Self {
        Self {
            state: UpdateState::Idle,
            check_job: Job::new(),
            install_job: Job::new(),
            log,
        }
    }

    pub fn current_version(&self) -> &'static str {
        CURRENT
    }

    /// Проверка в фоне. Вызывать при старте.
    pub fn check(&mut self) {
        if self.check_job.busy() || self.install_job.busy() {
            return;
        }
        self.state = UpdateState::Checking;
        self.check_job.spawn(|| z_core::updater::check(CURRENT));
    }

    pub fn is_available(&self) -> bool {
        matches!(self.state, UpdateState::Available(_))
    }

    /// Установка, только если проверка нашла новую версию.
    pub fn install(&mut self) {
        if !self.is_available() || self.install_job.busy() {
            return;
        }
        self.state = UpdateState::Installing;
        self.install_job.spawn(|| z_core::updater::install(CURRENT));
    }

    /// Скрыть баннер («Позже»).
    pub fn dismiss(&mut self) {
        if matches!(
            self.state,
            UpdateState::Available(_) | UpdateState::Failed(_)
        ) {
            self.state = UpdateState::Idle;
        }
    }

    /// Запускает новый экземпляр. После Ok вызови `cx.quit()`.
    pub fn restart(&self) -> std::io::Result<()> {
        std::process::Command::new(std::env::current_exe()?)
            .arg("--runtime")
            .arg(z_core::paths::runtime_root())
            .arg("--wait-for-pid")
            .arg(std::process::id().to_string())
            .spawn()?;
        Ok(())
    }

    pub fn poll(&mut self) {
        match self.check_job.poll() {
            JobPoll::Done(Ok(Some(v))) => {
                self.log.add(crate::i18n::tr_args(
                    "update-version-available",
                    &[("version", (v).to_string().into())],
                ));
                self.state = UpdateState::Available(v);
            }
            JobPoll::Done(Ok(None)) => self.state = UpdateState::Idle,
            JobPoll::Done(Err(e)) => {
                self.log.add(crate::i18n::tr_args(
                    "update-check-error",
                    &[("error", (e).to_string().into())],
                ));
                self.state = UpdateState::Failed(e);
            }
            JobPoll::Lost => {
                self.state = UpdateState::Failed(crate::i18n::tr("update-check-interrupted").into())
            }
            JobPoll::Pending => {}
        }
        match self.install_job.poll() {
            JobPoll::Done(Ok(v)) => {
                self.log.add(crate::i18n::tr_args(
                    "update-version-installed",
                    &[("version", (v).to_string().into())],
                ));
                self.state = UpdateState::Done(v);
            }
            JobPoll::Done(Err(e)) => {
                self.log.add(crate::i18n::tr_args(
                    "update-error",
                    &[("error", (e).to_string().into())],
                ));
                self.state = UpdateState::Failed(e);
            }
            JobPoll::Lost => {
                self.state = UpdateState::Failed(crate::i18n::tr("worker-interrupted").into())
            }
            JobPoll::Pending => {}
        }
    }
}
