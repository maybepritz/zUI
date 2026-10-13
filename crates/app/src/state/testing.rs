use std::time::{Duration, Instant};

use super::AppState;
use crate::{controllers::update::UpdateState, i18n::tr, notifications::EventKind};

pub(super) struct TestRun {
    previous: String,
    running: bool,
    automatic: bool,
    services: Vec<String>,
}

impl AppState {
    pub fn start_test(&mut self, automatic: bool) {
        if self.profile_busy()
            || self.diagnostics.busy()
            || self.zapret.installing()
            || matches!(self.update.state, UpdateState::Installing)
            || !z_core::dns::is_elevated()
        {
            return;
        }
        let run = TestRun {
            previous: self
                .zapret
                .selected()
                .map(|strategy| strategy.file_name.clone())
                .unwrap_or_default(),
            running: self.zapret.is_running(),
            automatic,
            services: self.settings.automation.services.clone(),
        };
        if automatic {
            let targets = z_core::services::targets(&self.settings.automation.services);
            self.diagnostics
                .start_with_targets(&mut self.zapret, targets);
        } else {
            self.diagnostics.start(&mut self.zapret);
        }
        if self.diagnostics.busy() {
            self.background.test = Some(run);
            self.background.next_retest = Instant::now() + Duration::from_secs(30 * 60);
            self.background.statistics.last_retest = Some(z_core::statistics::now());
            self.persist_statistics();
        }
    }

    pub(super) fn finish_test(&mut self, completed: bool) {
        if completed && self.diagnostics.results.iter().any(|score| score.is_ok()) {
            self.background.statistics.last_successful_test = Some(z_core::statistics::now());
        }
        let Some(run) = self.background.test.take() else {
            return;
        };
        let best = self.diagnostics.results.iter().find(|score| score.is_ok());
        let baseline = self
            .diagnostics
            .results
            .iter()
            .find(|score| score.file_name == run.previous);
        // Only switch on full reachability. If both work, require a 20% improvement.
        let better = best
            .filter(|best| {
                completed
                    && run.automatic
                    && run.services == self.settings.automation.services
                    && best.file_name != run.previous
                    && baseline.is_some_and(|old| {
                        !old.is_ok() || best.avg_ms.saturating_mul(5) < old.avg_ms.saturating_mul(4)
                    })
            })
            .map(|score| score.file_name.clone());
        // Disabling automatic switching while a test runs prevents applying its result.
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
            let restored = self.zapret.start();
            let message = if restored {
                tr("ui-could-not-start-the-selected-strategy-the")
            } else {
                tr("ui-could-not-restore-zapret-after-testing-see")
            };
            self.event(EventKind::Error, message.into());
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
            self.event(EventKind::Strategy, message.into());
        }
        self.background.services.reset();
        self.persist_statistics();
    }
}
