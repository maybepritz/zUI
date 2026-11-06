use std::time::{Duration, Instant};

use super::AppState;
use crate::{controllers::update::UpdateState, i18n::tr, notifications::EventKind};
use z_core::statistics::{StrategyResult, TestSummary};
use z_core::zapret::diagnostics::{DEFAULT_TARGETS, Options};

/// Зачем запущен тест стратегий.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    /// Пользователь проверяет все стратегии на вкладке «Стратегии».
    Manual,
    /// Фоновая перепроверка: переключиться можно, только если новая заметно лучше.
    Retest,
    /// «Подобрать»: остановиться на первой стратегии, с которой открывается всё,
    /// и включить с ней обход.
    Pick,
}

pub(super) struct TestRun {
    previous: String,
    running: bool,
    mode: TestMode,
    services: Vec<String>,
}

impl AppState {
    pub fn start_test(&mut self, mode: TestMode) {
        if self.diagnostics.busy()
            || self.zapret.installing()
            || matches!(self.update.state, UpdateState::Installing)
            || !z_core::dns::is_elevated()
        {
            return;
        }
        let services = self.settings.automation.services.clone();
        let previous = self
            .zapret
            .selected()
            .map(|strategy| strategy.file_name.clone())
            .unwrap_or_default();
        // Без выбранных сервисов, которые чинит zapret, проверяем стандартный набор адресов,
        // иначе тест молча не запустится.
        let mut targets = z_core::services::targets(&services);
        let video = targets.is_empty() || services.iter().any(|id| id == "youtube");
        if mode == TestMode::Manual || targets.is_empty() {
            targets = DEFAULT_TARGETS.iter().map(|url| url.to_string()).collect();
        }
        let mut strategies = self.zapret.strategies.clone();
        if mode == TestMode::Pick {
            // Сначала избранные, текущая — в конце: её подбирают, потому что она сломалась.
            let favorites = &self.settings.favorite_strategies;
            strategies
                .sort_by_key(|s| (s.file_name == previous, !favorites.contains(&s.file_name)));
        }
        let opts = Options {
            targets,
            stop_on_first_ok: mode == TestMode::Pick,
            video,
        };
        let run = TestRun {
            previous,
            running: self.zapret.is_running(),
            mode,
            services,
        };
        self.diagnostics.start(&mut self.zapret, strategies, opts);
        if self.diagnostics.busy() {
            self.background.test = Some(run);
            self.background.next_retest = Instant::now() + Duration::from_secs(30 * 60);
            self.background.statistics.last_retest = Some(z_core::clock::now());
            self.persist_statistics();
        }
    }

    /// Идёт подбор: главная показывает его вместо состояния обхода.
    pub fn picking(&self) -> bool {
        self.background
            .test
            .as_ref()
            .is_some_and(|run| run.mode == TestMode::Pick)
    }

    pub(super) fn finish_test(&mut self, completed: bool) {
        let Some(run) = self.background.test.take() else {
            return;
        };
        self.background.statistics.record_test(TestSummary {
            finished_at: z_core::clock::now(),
            completed,
            strategies_tested: self.diagnostics.results.len(),
            best: self.diagnostics.best().map(|score| StrategyResult {
                file_name: score.file_name.clone(),
                name: score.name.clone(),
                passed: score.passed,
                total: score.total,
                avg_ms: score.avg_ms,
            }),
        });
        if run.mode == TestMode::Pick && completed {
            self.finish_pick(&run);
        } else {
            self.finish_check(run, completed);
        }
        self.background.services.reset();
        self.persist_statistics();
    }

    /// Включает обход с первой стратегией, при которой всё открылось. Если такой
    /// нет, берёт ту, что открыла больше всего: это лучше, чем ничего.
    fn finish_pick(&mut self, run: &TestRun) {
        let full = self.diagnostics.results.iter().find(|s| s.is_ok());
        let chosen = full
            .or(self.diagnostics.best())
            .map(|s| (s.file_name.clone(), s.is_ok()));
        let Some((file, ok)) = chosen else {
            // Не открылось ничего: возвращаем как было.
            if let Some(index) = self.zapret.index_of(&run.previous) {
                self.select_strategy(index);
            }
            if run.running {
                self.zapret.start();
            }
            self.event(EventKind::Error, tr("pick-nothing-found"));
            return;
        };
        if let Some(index) = self.zapret.index_of(&file) {
            self.select_strategy(index);
        }
        if !self.zapret.is_running() && !self.zapret.start() {
            self.event(EventKind::Error, self.zapret.status.to_string());
            return;
        }
        let key = if ok { "pick-found" } else { "pick-partial" };
        self.event(EventKind::Strategy, tr(key));
    }

    fn finish_check(&mut self, run: TestRun, completed: bool) {
        let best = self.diagnostics.results.iter().find(|score| score.is_ok());
        let baseline = self
            .diagnostics
            .results
            .iter()
            .find(|score| score.file_name == run.previous);
        // Переключаемся только при полной доступности. Если работают обе, нужна выгода от 20%.
        let better = best
            .filter(|best| {
                completed
                    && run.mode == TestMode::Retest
                    && run.services == self.settings.automation.services
                    && best.file_name != run.previous
                    && baseline.is_some_and(|old| {
                        !old.is_ok() || best.avg_ms.saturating_mul(5) < old.avg_ms.saturating_mul(4)
                    })
            })
            .map(|score| score.file_name.clone());
        // Если во время теста выключить автопереключение, его результат не применяется.
        let replacement = better
            .as_ref()
            .filter(|_| self.settings.automation.auto_switch)
            .cloned();
        let target = replacement.as_deref().unwrap_or(&run.previous);
        if let Some(index) = self.zapret.index_of(target) {
            self.select_strategy(index);
        }

        if run.running && !self.zapret.start() {
            if let Some(index) = self.zapret.index_of(&run.previous) {
                self.select_strategy(index);
            }
            let message = if self.zapret.start() {
                tr("ui-could-not-start-the-selected-strategy-the")
            } else {
                tr("ui-could-not-restore-zapret-after-testing-see")
            };
            self.event(EventKind::Error, message);
        } else {
            let message = if replacement.is_some() {
                tr("ui-a-better-strategy-was-found-and-applied")
            } else if better.is_some() {
                tr("ui-a-better-strategy-is-available-on-the")
            } else if completed {
                tr("ui-test-complete-current-strategy-retained")
            } else {
                tr("ui-test-cancelled-previous-state-restored")
            };
            self.event(EventKind::Strategy, message);
        }
    }
}
