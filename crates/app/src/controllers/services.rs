use super::job::{Job, JobPoll};
use std::time::{Duration, Instant};
use z_core::{
    recovery::FailureTracker,
    services::{self, CheckBatch},
};

pub struct ServicesController {
    job: Job<(u64, Result<CheckBatch, String>)>,
    generation: u64,
    pub result: Option<CheckBatch>,
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

    pub fn poll(&mut self, ids: &[String], enabled: bool) {
        match self.job.poll() {
            JobPoll::Done((generation, _)) if generation != self.generation => {}
            JobPoll::Done((_, Ok(result))) => {
                self.degraded = self.tracker.observe(&result);
                self.result = Some(result);
            }
            JobPoll::Done((_, Err(error))) => {
                self.error = Some(error);
                self.degraded = false;
            }
            JobPoll::Lost => {
                self.error = Some(crate::i18n::tr("services-interrupted").into());
                self.degraded = false;
            }
            JobPoll::Pending => {}
        }
        if enabled && Instant::now() >= self.next_check {
            self.check(ids);
        }
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
