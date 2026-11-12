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
    /// Идёт фоновая повторная проверка: её сбой не показывается как ошибка.
    quiet: bool,
    log: Log,
}

impl UpdateController {
    pub fn new(log: Log) -> Self {
        Self {
            state: UpdateState::Idle,
            check_job: Job::new(),
            install_job: Job::new(),
            quiet: false,
            log,
        }
    }

    pub fn current_version(&self) -> &'static str {
        CURRENT
    }

    /// Проверка в фоне. Вызывать при старте и по кнопке.
    pub fn check(&mut self) {
        self.begin_check(false, || z_core::updater::check(CURRENT));
    }

    /// Тихая повторная проверка для приложения, которое неделями живёт в трее.
    /// Состояние меняется, только если нашлась новая версия: отсутствие сети
    /// не превращается в ошибку на главной и в уведомление.
    pub fn check_background(&mut self) {
        if matches!(self.state, UpdateState::Idle | UpdateState::Failed(_)) {
            self.begin_check(true, || z_core::updater::check(CURRENT));
        }
    }

    fn begin_check(
        &mut self,
        quiet: bool,
        check: impl FnOnce() -> Result<Option<String>, String> + Send + 'static,
    ) {
        if self.check_job.busy() || self.install_job.busy() {
            return;
        }
        self.quiet = quiet;
        if !quiet {
            self.state = UpdateState::Checking;
        }
        self.check_job.spawn(check);
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
                    &[("version", v.to_string().into())],
                ));
                self.state = UpdateState::Available(v);
            }
            JobPoll::Done(Ok(None)) if self.quiet => {}
            JobPoll::Done(Ok(None)) => self.state = UpdateState::Idle,
            JobPoll::Done(Err(e)) => {
                self.log.add(crate::i18n::tr_args(
                    "update-check-error",
                    &[("error", e.to_string().into())],
                ));
                if !self.quiet {
                    self.state = UpdateState::Failed(e);
                }
            }
            JobPoll::Lost if self.quiet => {}
            JobPoll::Lost => {
                self.state = UpdateState::Failed(crate::i18n::tr("update-check-interrupted"))
            }
            JobPoll::Pending => {}
        }
        match self.install_job.poll() {
            JobPoll::Done(Ok(v)) => {
                self.log.add(crate::i18n::tr_args(
                    "update-version-installed",
                    &[("version", v.to_string().into())],
                ));
                self.state = UpdateState::Done(v);
            }
            JobPoll::Done(Err(e)) => {
                self.log.add(crate::i18n::tr_args(
                    "update-error",
                    &[("error", e.to_string().into())],
                ));
                self.state = UpdateState::Failed(e);
            }
            JobPoll::Lost => {
                self.state = UpdateState::Failed(crate::i18n::tr("worker-interrupted"))
            }
            JobPoll::Pending => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finish(controller: &mut UpdateController) {
        for _ in 0..500 {
            controller.poll();
            if !controller.check_job.busy() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        panic!("check did not finish");
    }

    #[test]
    fn startup_check_reports_failure() {
        let mut controller = UpdateController::new(Log::new());
        controller.begin_check(false, || Err("offline".into()));
        assert!(matches!(controller.state, UpdateState::Checking));
        finish(&mut controller);
        assert!(matches!(controller.state, UpdateState::Failed(_)));
    }

    #[test]
    fn background_check_stays_silent_until_it_finds_a_version() {
        let mut controller = UpdateController::new(Log::new());
        controller.begin_check(true, || Err("offline".into()));
        assert!(matches!(controller.state, UpdateState::Idle));
        finish(&mut controller);
        assert!(matches!(controller.state, UpdateState::Idle));

        controller.begin_check(true, || Ok(None));
        finish(&mut controller);
        assert!(matches!(controller.state, UpdateState::Idle));

        controller.begin_check(true, || Ok(Some("9.9.9".into())));
        finish(&mut controller);
        assert!(matches!(&controller.state, UpdateState::Available(v) if v == "9.9.9"));
    }

    #[test]
    fn background_check_does_not_replace_a_found_update() {
        let mut controller = UpdateController::new(Log::new());
        controller.state = UpdateState::Available("1.0.0".into());
        controller.check_background();
        assert!(!controller.check_job.busy());
    }
}
