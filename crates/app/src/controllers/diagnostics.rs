use super::{
    job::{Job, JobPoll},
    zapret::ZapretController,
};
use std::sync::{Arc, atomic::Ordering};
use z_core::{
    log::Log,
    zapret::diagnostics::{self, Options, Progress, Score},
};

pub struct DiagnosticsController {
    job: Job<Vec<Score>>,
    progress: Arc<Progress>,
    total: usize,
    pub results: Vec<Score>,
    pub stop_on_first_ok: bool,
    log: Log,
}

impl DiagnosticsController {
    pub fn new(log: Log) -> Self {
        Self {
            job: Job::new(),
            progress: Arc::new(Progress::default()),
            total: 0,
            results: Vec::new(),
            stop_on_first_ok: false,
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

    pub fn start(&mut self, zapret: &mut ZapretController) {
        self.start_with_targets(zapret, Options::default().targets);
    }

    pub fn start_with_targets(&mut self, zapret: &mut ZapretController, targets: Vec<String>) {
        if self.busy() || zapret.installing() || zapret.strategies.is_empty() || targets.is_empty() {
            return;
        }
        if !zapret.lock() {
            self.log.add("Тест стратегий отменён: не удалось остановить Zapret");
            return;
        }
        self.progress = Arc::new(Progress::default());
        self.total = zapret.strategies.len();
        self.results.clear();

        let opts = Options {
            stop_on_first_ok: self.stop_on_first_ok,
            targets,
        };
        let dir = zapret.runtime_dir.clone();
        let strategies = zapret.strategies.clone();
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
                self.log.add(format!(
                    "Тест стратегий завершён: проверено {}",
                    scores.len()
                ));
                self.results = scores;
                zapret.unlock();
                return Some(!self.progress.cancel.load(Ordering::SeqCst));
            }
            JobPoll::Lost => {
                self.log.add("Тест стратегий прерван");
                zapret.unlock(); // иначе Zapret остался бы заблокированным
                return Some(false);
            }
            JobPoll::Pending => {}
        }
        None
    }
}
