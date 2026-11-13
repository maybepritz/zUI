use crate::services::{Availability, CheckBatch};
use std::collections::{HashMap, HashSet};
/// Нужны три подряд сетевых сбоя у сервиса, который раньше открывался.
/// Проверки HTTP (капча) и пропажа интернета тест не запускают.

#[derive(Default)]
pub struct FailureTracker {
    reachable: HashSet<String>,
    failures: HashMap<String, u8>,
}

impl FailureTracker {
    pub fn observe(&mut self, batch: &CheckBatch) -> bool {
        if !batch.internet_available {
            self.failures.clear();
            return false;
        }
        let mut degraded = false;
        for check in &batch.checks {
            // Telegram и сервисы через VPN идут мимо zapret: менять стратегию из-за
            // их сбоев бессмысленно.
            if check.id == crate::services::TELEGRAM || check.via_vpn {
                self.failures.remove(&check.id);
                continue;
            }
            match check.availability {
                Availability::Reachable => {
                    self.reachable.insert(check.id.clone());
                    self.failures.remove(&check.id);
                }
                Availability::NetworkError if self.reachable.contains(&check.id) => {
                    let count = self.failures.entry(check.id.clone()).or_default();
                    *count = count.saturating_add(1);
                    degraded |= *count >= 3;
                }
                _ => {
                    self.failures.remove(&check.id);
                }
            }
        }
        degraded
    }
}
/// Ограниченные повторы не дают уйти в цикл падений. Минута стабильной работы сбрасывает счётчик.

#[derive(Default)]
pub struct RestartBudget {
    attempts: u8,
    last_attempt: Option<u64>,
}

impl RestartBudget {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn allow(&mut self, now: u64) -> bool {
        if self.attempts >= 3 {
            return false;
        }
        let delay = [5, 15, 60][self.attempts as usize];
        if self
            .last_attempt
            .is_some_and(|last| now.saturating_sub(last) < delay)
        {
            return false;
        }
        self.attempts += 1;
        self.last_attempt = Some(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::Check;
    #[test]
    fn only_repeated_transport_failures_trigger_recovery() {
        let mut tracker = FailureTracker::default();
        let mut batch = CheckBatch {
            checks: vec![Check {
                id: "chatgpt".into(),
                availability: Availability::Reachable,
                latency_ms: 1,
                via_vpn: false,
            }],
            internet_available: true,
            checked_at: 0,
            vpn: Default::default(),
            telegram_client: None,
        };
        assert!(!tracker.observe(&batch));
        batch.checks[0].availability = Availability::Http(403);
        for _ in 0..4 {
            assert!(!tracker.observe(&batch));
        }
        batch.checks[0].availability = Availability::NetworkError;
        assert!(!tracker.observe(&batch));
        assert!(!tracker.observe(&batch));
        assert!(tracker.observe(&batch));
        batch.internet_available = false;
        assert!(!tracker.observe(&batch));
    }

    #[test]
    fn failures_through_vpn_do_not_trigger_recovery() {
        let mut tracker = FailureTracker::default();
        let mut batch = CheckBatch {
            checks: vec![Check {
                id: "youtube".into(),
                availability: Availability::Reachable,
                latency_ms: 1,
                via_vpn: false,
            }],
            internet_available: true,
            checked_at: 0,
            vpn: Default::default(),
            telegram_client: None,
        };
        assert!(!tracker.observe(&batch));
        batch.checks[0].availability = Availability::NetworkError;
        batch.checks[0].via_vpn = true;
        for _ in 0..5 {
            assert!(!tracker.observe(&batch));
        }
    }
    #[test]
    fn restart_budget_limits_crash_loops() {
        let mut budget = RestartBudget::default();
        assert!(budget.allow(100));
        assert!(!budget.allow(101));
        assert!(budget.allow(115));
        assert!(budget.allow(175));
        assert!(!budget.allow(1000));
        budget.reset();
        assert!(budget.allow(1001));
    }
}
