use super::job::{Job, JobPoll};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use z_core::{
    recovery::FailureTracker,
    services::{self, CheckBatch},
};

pub struct ServicesController {
    job: Job<(u64, Result<CheckBatch, String>)>,
    generation: u64,
    pub result: Option<CheckBatch>,
    /// Проверки за последний час для живого графика на главной. Только в памяти:
    /// после перезапуска час начинается заново, суточная история лежит в `Health`.
    recent: VecDeque<CheckBatch>,
    pub error: Option<String>,
    tracker: FailureTracker,
    next_check: Instant,
    pub degraded: bool,
}

impl ServicesController {
    pub fn new() -> Self {
        Self {
            job: Job::new(),
            generation: 0,
            result: None,
            recent: VecDeque::new(),
            error: None,
            tracker: FailureTracker::default(),
            next_check: Instant::now(),
            degraded: false,
        }
    }

    pub fn busy(&self) -> bool {
        self.job.busy()
    }

    pub fn check(&mut self, ids: &[String]) {
        if self.busy() || ids.is_empty() {
            return;
        }
        let ids = ids.to_vec();
        let generation = self.generation;
        self.error = None;
        self.next_check = Instant::now() + Duration::from_secs(60);
        self.job.spawn(move || {
            (
                generation,
                services::check(&ids).map_err(|e| format!("{e:#}")),
            )
        });
    }

    /// Возвращает `true`, когда пришёл новый результат проверки.
    pub fn poll(&mut self, ids: &[String], enabled: bool) -> bool {
        let mut fresh = false;
        match self.job.poll() {
            JobPoll::Done((generation, _)) if generation != self.generation => {}
            JobPoll::Done((_, Ok(result))) => {
                self.degraded = self.tracker.observe(&result);
                remember(&mut self.recent, result.clone());
                self.result = Some(result);
                fresh = true;
            }
            JobPoll::Done((_, Err(error))) => {
                self.error = Some(error);
                self.degraded = false;
            }
            JobPoll::Lost => {
                self.error = Some(crate::i18n::tr("services-interrupted"));
                self.degraded = false;
            }
            JobPoll::Pending => {}
        }
        if enabled && Instant::now() >= self.next_check {
            self.check(ids);
        }
        fresh
    }

    pub fn reset(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.tracker = FailureTracker::default();
        self.degraded = false;
        self.result = None;
        self.error = None;
        self.next_check = Instant::now() + Duration::from_secs(60);
    }
}

/// Сколько секунд проверок хранится для живого графика.
const RECENT_SECONDS: u64 = 60 * 60;

fn remember(recent: &mut VecDeque<CheckBatch>, batch: CheckBatch) {
    let oldest = batch.checked_at.saturating_sub(RECENT_SECONDS);
    while recent.front().is_some_and(|old| old.checked_at < oldest) {
        recent.pop_front();
    }
    recent.push_back(batch);
}

impl ServicesController {
    /// Точки одного сервиса за последний час: время проверки и задержка,
    /// `None` — сервис в эту проверку не открылся. Проверки без интернета пропущены:
    /// они говорят о сети, а не о сервисе.
    pub fn recent_points(&self, id: &str) -> Vec<(u64, Option<u64>)> {
        self.recent
            .iter()
            .filter(|batch| batch.internet_available)
            .filter_map(|batch| {
                let check = batch.checks.iter().find(|check| check.id == id)?;
                let latency = check.availability.connected().then_some(check.latency_ms);
                Some((batch.checked_at, latency))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z_core::services::{Availability, Check};

    fn batch(at: u64, availability: Availability, internet: bool) -> CheckBatch {
        CheckBatch {
            checks: vec![Check {
                id: "youtube".into(),
                availability,
                latency_ms: 40,
            }],
            internet_available: internet,
            checked_at: at,
        }
    }

    #[test]
    fn recent_checks_keep_only_the_last_hour() {
        let mut controller = ServicesController::new();
        for at in [100, 1_000, 3_800] {
            remember(
                &mut controller.recent,
                batch(at, Availability::Reachable, true),
            );
        }
        let times: Vec<u64> = controller
            .recent_points("youtube")
            .iter()
            .map(|(at, _)| *at)
            .collect();
        assert_eq!(times, [1_000, 3_800]);
    }

    #[test]
    fn failed_check_is_a_gap_and_offline_check_is_skipped() {
        let mut controller = ServicesController::new();
        remember(
            &mut controller.recent,
            batch(10, Availability::Reachable, true),
        );
        remember(
            &mut controller.recent,
            batch(70, Availability::NetworkError, true),
        );
        remember(
            &mut controller.recent,
            batch(130, Availability::NetworkError, false),
        );
        remember(
            &mut controller.recent,
            batch(190, Availability::Http(403), true),
        );
        assert_eq!(
            controller.recent_points("youtube"),
            [(10, Some(40)), (70, None), (190, Some(40))]
        );
        assert!(controller.recent_points("discord").is_empty());
    }
}
