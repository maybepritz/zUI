//! История доступности сервисов за последние сутки для графика на главной.
//!
//! Проверка идёт раз в минуту, но хранить каждую незачем: результаты сводятся
//! в 15-минутные корзины. Потеря файла безобидна, график просто начнётся заново.
use crate::services::CheckBatch;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

pub const BUCKET_SECONDS: u64 = 15 * 60;
/// Сутки 15-минутных корзин.
pub const DAY_BUCKETS: u64 = 96;

/// Итог проверок одного сервиса за 15 минут.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bucket {
    /// Начало окна, unix-секунды, кратно `BUCKET_SECONDS`.
    pub start: u64,
    pub ok: u16,
    pub failed: u16,
    /// Сумма задержек успешных проверок, мс.
    pub ms_total: u32,
}

impl Bucket {
    pub fn average_ms(&self) -> Option<u64> {
        (self.ok > 0).then(|| u64::from(self.ms_total) / u64::from(self.ok))
    }

    /// За всё окно сервис ни разу не открылся.
    pub fn down(&self) -> bool {
        self.ok == 0 && self.failed > 0
    }
}

/// Закончившийся сбой: сервис не открывался с `from` до `to`.
#[derive(Clone, Debug, PartialEq)]
pub struct Outage {
    pub service: String,
    pub from: u64,
    pub to: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Health {
    pub services: BTreeMap<String, Vec<Bucket>>,
    /// С какого момента сервис не открывается, если сбой ещё идёт.
    pub down_since: BTreeMap<String, u64>,
}

impl Health {
    /// Пустая история, если файла нет или он повреждён.
    pub fn load(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        crate::storage::save_json(path, self)
    }

    /// Последние сутки корзин сервиса, от старых к новым.
    pub fn day(&self, service: &str) -> &[Bucket] {
        self.services.get(service).map_or(&[], Vec::as_slice)
    }

    /// Добавляет результат проверки и возвращает закончившиеся сбои.
    /// Без интернета проверка не записывается: это не блокировка.
    pub fn record(&mut self, batch: &CheckBatch) -> Vec<Outage> {
        if !batch.internet_available {
            return Vec::new();
        }
        let now = batch.checked_at;
        let start = now - now % BUCKET_SECONDS;
        let mut ended = Vec::new();
        for check in &batch.checks {
            let buckets = self.services.entry(check.id.clone()).or_default();
            if buckets.last().is_none_or(|b| b.start != start) {
                buckets.push(Bucket {
                    start,
                    ..Bucket::default()
                });
            }
            let Some(bucket) = buckets.last_mut() else {
                continue;
            };
            if check.availability.connected() {
                bucket.ok = bucket.ok.saturating_add(1);
                let ms = u32::try_from(check.latency_ms).unwrap_or(u32::MAX);
                bucket.ms_total = bucket.ms_total.saturating_add(ms);
                if let Some(from) = self.down_since.remove(&check.id) {
                    ended.push(Outage {
                        service: check.id.clone(),
                        from,
                        to: now,
                    });
                }
            } else {
                bucket.failed = bucket.failed.saturating_add(1);
                self.down_since.entry(check.id.clone()).or_insert(now);
            }
        }
        self.forget_before(start.saturating_sub((DAY_BUCKETS - 1) * BUCKET_SECONDS));
        ended
    }

    fn forget_before(&mut self, oldest: u64) {
        for buckets in self.services.values_mut() {
            buckets.retain(|b| b.start >= oldest);
        }
        self.services.retain(|_, buckets| !buckets.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{Availability, Check};

    fn batch(at: u64, availability: Availability, internet: bool) -> CheckBatch {
        CheckBatch {
            checks: vec![Check {
                id: "youtube".into(),
                availability,
                latency_ms: 100,
                via_vpn: false,
            }],
            internet_available: internet,
            checked_at: at,
            vpn: Default::default(),
            telegram_client: None,
        }
    }

    #[test]
    fn checks_in_one_window_share_a_bucket() {
        let mut health = Health::default();
        health.record(&batch(900, Availability::Reachable, true));
        health.record(&batch(960, Availability::Http(403), true));
        health.record(&batch(1800, Availability::NetworkError, true));
        let day = health.day("youtube");
        assert_eq!(day.len(), 2);
        assert_eq!((day[0].ok, day[0].average_ms()), (2, Some(100)));
        assert!(day[1].down());
    }

    #[test]
    fn outage_ends_when_the_service_opens_again() {
        let mut health = Health::default();
        assert!(
            health
                .record(&batch(100, Availability::NetworkError, true))
                .is_empty()
        );
        assert!(
            health
                .record(&batch(160, Availability::NetworkError, true))
                .is_empty()
        );
        let ended = health.record(&batch(220, Availability::Reachable, true));
        assert_eq!(
            ended,
            vec![Outage {
                service: "youtube".into(),
                from: 100,
                to: 220
            }]
        );
    }

    #[test]
    fn no_internet_is_not_an_outage_and_old_buckets_go() {
        let mut health = Health::default();
        health.record(&batch(0, Availability::Reachable, true));
        health.record(&batch(60, Availability::NetworkError, false));
        assert!(health.down_since.is_empty());
        health.record(&batch(
            DAY_BUCKETS * BUCKET_SECONDS,
            Availability::Reachable,
            true,
        ));
        assert_eq!(health.day("youtube").len(), 1);
    }
}
