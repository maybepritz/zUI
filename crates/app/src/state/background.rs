use std::time::{Duration, Instant};

use z_core::{
    health::Health,
    recovery::RestartBudget,
    statistics::{self, Statistics, StatisticsEventKind},
};

use super::{AppState, testing::TestRun};
use crate::{
    controllers::{services::ServicesController, update::UpdateState},
    i18n::tr,
    notifications::EventKind,
};

const CHECK_INTERVAL: Duration = Duration::from_secs(60);
const RETEST_INTERVAL: u64 = 24 * 60 * 60;
/// Как часто работающее приложение само узнаёт о новых версиях zUI и Zapret.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

pub struct Background {
    pub services: ServicesController,
    pub statistics: Statistics,
    /// История доступности сервисов для графика на главной.
    pub health: Health,
    pub hidden: bool,
    pub(super) test: Option<TestRun>,
    pub(super) next_retest: Instant,
    restart_budget: RestartBudget,
    stable_since: Option<Instant>,
    last_tick: Instant,
    last_save: Instant,
    last_update: UpdateState,
    next_update_check: Instant,
}

impl Background {
    pub fn new(statistics: Statistics, health: Health) -> Self {
        let now = Instant::now();
        Self {
            services: ServicesController::new(),
            statistics,
            health,
            hidden: false,
            test: None,
            next_retest: now + CHECK_INTERVAL,
            restart_budget: RestartBudget::default(),
            stable_since: None,
            last_tick: now,
            last_save: now,
            last_update: UpdateState::Idle,
            next_update_check: now + UPDATE_CHECK_INTERVAL,
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
        // При старте проверку запускает prepare; дальше повторяем её, пока приложение живёт в трее.
        if Instant::now() >= self.background.next_update_check {
            self.background.next_update_check = Instant::now() + UPDATE_CHECK_INTERVAL;
            self.update.check_background();
            self.zapret.check_update();
        }
        self.poll_update_events();

        let checking_allowed = self.settings.onboarded
            && !self.diagnostics.busy()
            && !self.zapret.installing()
            && !matches!(self.update.state, UpdateState::Installing);
        // Новый процесс winws (включение, смена стратегии, перезапуск) — проверяем заново.
        let running = self.zapret.is_running();
        self.background
            .services
            .follow(self.zapret.pid().filter(|_| running));
        let telegram = self.telegram.proxy.check_target();
        self.background.services.follow_telegram(telegram.is_some());
        // Проверяем, пока работает обход или Telegram-прокси (для главной)
        // или включено автопереключение.
        let enabled = running || telegram.is_some() || self.settings.automation.auto_switch;
        let fresh = self.background.services.poll(
            &self.settings.automation.services,
            telegram,
            checking_allowed && enabled,
        );
        if fresh {
            self.record_health();
        }
        self.poll_watchdog();

        let daily_due = self
            .background
            .statistics
            .last_retest
            .is_none_or(|last| z_core::clock::now().saturating_sub(last) >= RETEST_INTERVAL);
        let preferences = &self.settings.automation;
        let should_retest = (preferences.auto_switch && self.background.services.degraded)
            || (preferences.daily_retest && daily_due);
        if checking_allowed
            && should_retest
            && Instant::now() >= self.background.next_retest
            && self.zapret.is_running()
        {
            self.start_test(super::TestMode::Retest);
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
            let since = self
                .background
                .stable_since
                .get_or_insert_with(Instant::now);
            if since.elapsed() >= CHECK_INTERVAL {
                self.background.restart_budget.reset();
            }
            return;
        }
        self.background.stable_since = None;
        if !self.background.restart_budget.allow(z_core::clock::now()) {
            return;
        }
        let restarted = self.zapret.start();
        if restarted {
            self.background.statistics.watchdog_restarts = self
                .background
                .statistics
                .watchdog_restarts
                .saturating_add(1);
        }
        self.background
            .statistics
            .record_event(statistics::StatisticsEventKind::Recovery { restarted });
        let message = if restarted {
            tr("ui-winws-exited-and-was-restarted")
        } else {
            tr("ui-could-not-restart-winws-automatic-retries-are")
        };
        self.event(EventKind::Error, message);
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
                crate::i18n::tr_args("update-version-available", &[("version", version.into())]),
            ),
            UpdateState::Done(version) => self.event(
                EventKind::Update,
                crate::i18n::tr_args("update-ready-version", &[("version", version.into())]),
            ),
            UpdateState::Failed(error) => self.event(EventKind::Error, error),
            _ => {}
        }
    }

    /// Записывает свежую проверку в историю, а закончившиеся сбои — в события.
    fn record_health(&mut self) {
        let Some(batch) = &self.background.services.result else {
            return;
        };
        for outage in self.background.health.record(batch) {
            self.background
                .statistics
                .record_event(StatisticsEventKind::ServiceDown {
                    service: outage.service,
                    from: outage.from,
                    to: outage.to,
                });
        }
    }

    pub(super) fn persist_statistics(&mut self) {
        self.background.record_uptime();
        self.background.last_save = Instant::now();
        let path = self.data_dir().join("statistics.json");
        if let Err(error) = self.background.statistics.save(&path) {
            self.log.add(format!("Statistics: {error:#}"));
        }
        let path = self.data_dir().join("health.json");
        if let Err(error) = self.background.health.save(&path) {
            self.log.add(format!("Health: {error:#}"));
        }
    }
}
