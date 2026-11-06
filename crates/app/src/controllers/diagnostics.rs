use super::{
    job::{Job, JobPoll},
    zapret::ZapretController,
};
use std::sync::{Arc, atomic::Ordering};
use z_core::{
    log::Log,
    zapret::{
        Strategy,
        diagnostics::{self, Options, Progress, Score},
    },
};

pub struct DiagnosticsController {
    job: Job<Vec<Score>>,
    progress: Arc<Progress>,
    total: usize,
    pub results: Vec<Score>,
    log: Log,
}

impl DiagnosticsController {
    pub fn new(log: Log) -> Self {
        Self {
            job: Job::new(),
            progress: Arc::new(Progress::default()),
            total: 0,
            results: Vec::new(),
            log,
        }
    }

    pub fn busy(&self) -> bool {
        self.job.busy()
    }

    /// (проверено, всего)
    pub fn progress(&self) -> (usize, usize) {
        (self.progress.done.load(Ordering::SeqCst), self.total)
    }

    /// Во время теста живые результаты, после него итоговые.
    pub fn rows(&self) -> Vec<Score> {
        if self.busy() {
            self.progress
                .live
                .lock()
                .map(|l| l.clone())
                .unwrap_or_default()
        } else {
            self.results.clone()
        }
    }

    /// Лучшая: итог уже отсортирован (больше целей, меньше задержка).
    pub fn best(&self) -> Option<&Score> {
        self.results
            .iter()
            .find(|s| s.passed > 0 && s.error.is_none())
    }

    /// Индекс лучшей стратегии в списке контроллера Zapret.
    pub fn best_index(&self, zapret: &ZapretController) -> Option<usize> {
        self.best().and_then(|s| zapret.index_of(&s.file_name))
    }

    /// Проверяет `strategies` в этом порядке. Обычный Zapret на время теста останавливается.
    pub fn start(
        &mut self,
        zapret: &mut ZapretController,
        strategies: Vec<Strategy>,
        opts: Options,
    ) {
        if self.busy() || zapret.installing() || strategies.is_empty() || opts.targets.is_empty() {
            return;
        }
        if !zapret.lock() {
            self.log.add(crate::i18n::tr("test-stop-failed"));
            return;
        }
        self.progress = Arc::new(Progress::default());
        self.total = strategies.len();
        self.results.clear();

        let dir = zapret.runtime_dir.clone();
        let (log, progress) = (self.log.clone(), self.progress.clone());
        self.job
            .spawn(move || diagnostics::run(&dir, strategies, opts, log, &progress));
    }

    pub fn cancel(&self) {
        self.progress.cancel.store(true, Ordering::SeqCst);
    }

    pub fn shutdown(&mut self) {
        self.cancel();
        self.job.wait();
    }

    pub fn poll(&mut self, zapret: &mut ZapretController) -> Option<bool> {
        match self.job.poll() {
            JobPoll::Done(scores) => {
                self.log.add(crate::i18n::tr_args(
                    "test-complete-count",
                    &[("count", scores.len().into())],
                ));
                self.results = scores;
                zapret.unlock();
                return Some(!self.progress.cancel.load(Ordering::SeqCst));
            }
            JobPoll::Lost => {
                self.log.add(crate::i18n::tr("test-interrupted"));
                zapret.unlock(); // иначе Zapret остался бы заблокированным
                return Some(false);
            }
            JobPoll::Pending => {}
        }
        None
    }
}
