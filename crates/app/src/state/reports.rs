//! Каталог данных и отчёты с очищенными журналами.

use super::AppState;
use crate::{i18n::tr, notifications::EventKind};
use std::path::PathBuf;

impl AppState {
    pub fn data_dir(&self) -> PathBuf {
        self.runtime_dir
            .parent()
            .unwrap_or(&self.runtime_dir)
            .to_path_buf()
    }

    pub fn issue_report(&self) -> String {
        let tests = self
            .diagnostics
            .results
            .iter()
            .map(|score| {
                format!(
                    "{}: {}/{}; {} ms; failed={:?}; error={:?}",
                    score.file_name,
                    score.passed,
                    score.total,
                    score.avg_ms,
                    score.failed,
                    score.error,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let summary = format!(
            "Administrator: {}\nStrategy: {}\nZapret: {}\nTelegram: {}\nServices: {:?}\nTests: {}\nStatistics: {:?}\nDNS: {}\nTest results:\n{tests}",
            z_core::dns::is_elevated(),
            self.settings.selected_strategy,
            self.zapret.status,
            self.telegram.proxy.status,
            self.background.services.result,
            self.diagnostics.results.len(),
            self.background.statistics,
            self.dns.active_name,
        );
        z_core::report::build(env!("CARGO_PKG_VERSION"), &summary, &self.log.get())
    }

    pub fn export_report(&mut self) -> Option<PathBuf> {
        match z_core::report::save(&self.data_dir().join("reports"), &self.issue_report()) {
            Ok(path) => {
                self.settings_status = tr("ui-report-saved-review-it-before-publishing").into();
                Some(path)
            }
            Err(error) => {
                self.event(EventKind::Error, error.to_string());
                None
            }
        }
    }
}
