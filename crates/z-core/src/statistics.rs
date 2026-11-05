//! Сохраняемые счётчики, последний результат диагностики и ограниченная история событий.
use crate::settings::Recovery;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const HISTORY_LIMIT: usize = 100;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Сохранённый результат диагностики; текущее состояние связи он не описывает.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestSummary {
    pub finished_at: u64,
    pub completed: bool,
    pub strategies_tested: usize,
    pub best: Option<StrategyResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StrategyResult {
    pub file_name: String,
    pub name: String,
    pub passed: usize,
    pub total: usize,
    pub avg_ms: u64,
}

impl StrategyResult {
    pub fn fully_available(&self) -> bool {
        self.total > 0 && self.passed == self.total
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatisticsEvent {
    pub timestamp: u64,
    pub kind: StatisticsEventKind,
}

/// Храним смысл, а не текст, чтобы историю можно было перевести при отрисовке.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StatisticsEventKind {
    StrategyChanged {
        strategy: String,
    },
    TestFinished {
        completed: bool,
    },
    Recovery {
        restarted: bool,
    },
    /// Сервис не открывался с `from` до `to`, unix-секунды.
    ServiceDown {
        service: String,
        from: u64,
        to: u64,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Statistics {
    pub uptime_seconds: u64,
    pub strategy_changes: u64,
    pub watchdog_restarts: u64,
    pub last_successful_test: Option<u64>,
    pub last_retest: Option<u64>,
    pub last_test: Option<TestSummary>,
    pub history: Vec<StatisticsEvent>,
}

impl Statistics {
    /// Загружает статистику; нечитаемый файл откладывается, пробуется резервная копия,
    /// и только потом пустая статистика.
    pub fn load(path: &Path) -> (Self, Option<Recovery>) {
        crate::settings::load_recovering(path, Self::read)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        crate::settings::save_with_backup(path, self, Self::read)
    }

    /// `Ok(None)`, если файла ещё нет.
    fn read(path: &Path) -> anyhow::Result<Option<Self>> {
        match std::fs::read(path) {
            Ok(bytes) => {
                let mut statistics: Self = serde_json::from_slice(&bytes)?;
                statistics.trim_history();
                Ok(Some(statistics))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn record_event(&mut self, kind: StatisticsEventKind) {
        self.history.push(StatisticsEvent {
            timestamp: now(),
            kind,
        });
        self.trim_history();
    }

    pub fn record_test(&mut self, summary: TestSummary) {
        if summary.completed
            && summary
                .best
                .as_ref()
                .is_some_and(StrategyResult::fully_available)
        {
            self.last_successful_test = Some(summary.finished_at);
        }
        self.record_event(StatisticsEventKind::TestFinished {
            completed: summary.completed,
        });
        self.last_test = Some(summary);
    }

    fn trim_history(&mut self) {
        let excess = self.history.len().saturating_sub(HISTORY_LIMIT);
        self.history.drain(..excess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_statistics_remain_readable() {
        let old = r#"{"uptime_seconds":123,"strategy_changes":4,"last_successful_test":42}"#;
        let stats: Statistics = serde_json::from_str(old).unwrap();
        assert_eq!(stats.uptime_seconds, 123);
        assert_eq!(stats.last_successful_test, Some(42));
        assert!(stats.last_test.is_none());
        assert!(stats.history.is_empty());
    }

    #[test]
    fn history_keeps_the_newest_events_and_round_trips() {
        let mut stats = Statistics::default();
        for index in 0..HISTORY_LIMIT + 3 {
            stats.record_event(StatisticsEventKind::StrategyChanged {
                strategy: index.to_string(),
            });
        }
        let restored: Statistics =
            serde_json::from_str(&serde_json::to_string(&stats).unwrap()).unwrap();
        assert_eq!(restored.history.len(), HISTORY_LIMIT);
        assert!(matches!(&restored.history[0].kind,
            StatisticsEventKind::StrategyChanged { strategy } if strategy == "3"));
    }

    #[test]
    fn cancelled_or_partial_tests_do_not_replace_last_success_time() {
        let mut stats = Statistics::default();
        for (completed, passed, expected) in [
            (true, 2, Some(100)),
            (false, 2, Some(100)),
            (true, 1, Some(100)),
        ] {
            stats.record_test(TestSummary {
                finished_at: 100 + stats.history.len() as u64,
                completed,
                strategies_tested: 1,
                best: Some(StrategyResult {
                    file_name: "test.bat".into(),
                    name: "Test".into(),
                    passed,
                    total: 2,
                    avg_ms: 15,
                }),
            });
            assert_eq!(stats.last_successful_test, expected);
        }
        assert_eq!(
            stats
                .last_test
                .as_ref()
                .unwrap()
                .best
                .as_ref()
                .unwrap()
                .passed,
            1
        );
    }
}
