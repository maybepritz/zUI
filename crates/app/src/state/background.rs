use std::time::{Duration, Instant};

use z_core::{
    recovery::RestartBudget,
    statistics::{self, Statistics},
};

use super::{AppState, testing::TestRun};
use crate::{
    controllers::{services::ServicesController, update::UpdateState},
    i18n::tr,
    notifications::EventKind,
};

const CHECK_INTERVAL: Duration = Duration::from_secs(60);
const RETEST_INTERVAL: u64 = 24 * 60 * 60;

pub struct Background {
    pub services: ServicesController,
    pub statistics: Statistics,
    pub hidden: bool,
    pub(super) test: Option<TestRun>,
    pub(super) next_retest: Instant,
    restart_budget: RestartBudget,
    stable_since: Option<Instant>,
    last_tick: Instant,
    last_save: Instant,
    last_update: UpdateState,
}

impl Background {
    pub fn new(statistics: Statistics) -> Self {
        let now = Instant::now();
        Self {
            services: ServicesController::new(),
            statistics,
            hidden: false,
            test: None,
            next_retest: now + CHECK_INTERVAL,
            restart_budget: RestartBudget::default(),
            stable_since: None,
            last_tick: now,
            last_save: now,
            last_update: UpdateState::Idle,
        }
    }

    fn record_uptime(&mut self) {
        let elapsed = self.last_tick.elapsed().as_secs();
        self.statistics.uptime_seconds = self.statistics.uptime_seconds.saturating_add(elapsed);
        self.last_tick += Duration::from_secs(elapsed);
    }
}

impl AppState {
    pub(super) fn poll_background(&mut self) {
        self.background.record_uptime();
        if self.background.last_save.elapsed() >= CHECK_INTERVAL {
            self.persist_statistics();
        }
        self.poll_update_events();

        let checking_allowed = self.settings.onboarded
            && !self.diagnostics.busy()
            && !self.zapret.installing()
            && !matches!(self.update.state, UpdateState::Installing);
        self.background.services.poll(
            &self.settings.automation.services,
            checking_allowed && self.settings.automation.auto_switch,
        );
        self.poll_watchdog();

        let daily_due = self.background.statistics.last_retest.is_none_or(|last| {
            statistics::now().saturating_sub(last) >= RETEST_INTERVAL
        });
        let preferences = &self.settings.automation;
        let should_retest = (preferences.auto_switch && self.background.services.degraded)
            || (preferences.daily_retest && daily_due);
        if checking_allowed
            && should_retest
            && Instant::now() >= self.background.next_retest
            && self.zapret.is_running()
        {
            self.start_test(true);
        }
    }

    fn poll_watchdog(&mut self) {
        if !self.settings.automation.watchdog
            || !self.zapret.wanted_running
            || self.zapret.locked
            || self.zapret.installing()
        {
            self.background.restart_budget.reset();
            self.background.stable_since = None;
            return;
        }
        if self.zapret.is_running() {
            let since = self.background.stable_since.get_or_insert_with(Instant::now);
            if since.elapsed() >= CHECK_INTERVAL {
                self.background.restart_budget.reset();
            }
            return;
        }
        self.background.stable_since = None;
        if !self.background.restart_budget.allow(statistics::now()) {
            return;
        }
        let restarted = self.zapret.start();
        if restarted {
            self.background.statistics.watchdog_restarts =
                self.background.statistics.watchdog_restarts.saturating_add(1);
        }
        let message = if restarted {
            tr("winws завершился и был перезапущен", "winws exited and was restarted")
        } else {
            tr(
                "Не удалось перезапустить winws. Автоповторы ограничены тремя попытками",
                "Could not restart winws. Automatic retries are limited to three attempts",
            )
        };
        self.event(EventKind::Error, message.into());
        self.persist_statistics();
    }

    fn poll_update_events(&mut self) {
        if self.update.state == self.background.last_update {
            return;
        }
        self.background.last_update = self.update.state.clone();
        match self.update.state.clone() {
            UpdateState::Available(version) => self.event(
                EventKind::Update,
                format!("{}: {version}", tr("Доступно обновление", "Update available")),
            ),
            UpdateState::Done(version) => self.event(
                EventKind::Update,
                format!(
                    "{}: {version}",
                    tr(
                        "Обновление готово. Перезапустите приложение",
                        "Update ready. Restart the app",
                    ),
                ),
            ),
            UpdateState::Failed(error) => self.event(EventKind::Error, error),
            _ => {}
        }
    }

    pub(super) fn persist_statistics(&mut self) {
        self.background.record_uptime();
        self.background.last_save = Instant::now();
        let path = self.data_dir().join("statistics.json");
        if let Err(error) = self.background.statistics.save(&path) {
            self.log.add(format!("Statistics: {error:#}"));
        }
    }
}
