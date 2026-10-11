use super::AppState;
use crate::{i18n::tr, notifications::{self, EventKind}, platform};
use std::path::PathBuf;

impl AppState {
    pub fn data_dir(&self) -> PathBuf {
        self.runtime_dir.parent().unwrap_or(&self.runtime_dir).to_path_buf()
    }

    pub fn event(&mut self, kind: EventKind, message: String) {
        self.log.add(&message);
        notifications::show(&self.settings.notifications, kind, self.background.hidden, &message);
        self.settings_status = message.into();
    }

    pub fn set_autostart(&mut self, enabled: bool) {
        match platform::set_autostart(enabled) {
            Ok(()) => {
                self.settings.autostart = enabled;
                self.save_settings();
            }
            Err(error) => self.event(EventKind::Error, error),
        }
    }

    pub fn elevate(&mut self) -> bool {
        if !self.save_settings() {
            return false;
        }
        match platform::relaunch_elevated() {
            Ok(()) => true,
            Err(error) => {
                self.event(EventKind::Error, error);
                false
            }
        }
    }

    pub fn start_selected_services(&mut self) {
        if self.settings.startup.zapret && !self.zapret.is_running() && !self.zapret.start() {
            self.event(EventKind::Error, self.zapret.status.to_string());
        }
        if self.settings.startup.telegram && !self.telegram.running() {
            self.telegram.toggle();
        }
    }

    pub fn complete_onboarding(&mut self) -> bool {
        self.settings.onboarded = true;
        self.settings.last_seen_version = env!("CARGO_PKG_VERSION").into();
        if let Err(error) = self.settings.save(&self.data_dir().join("settings.json")) {
            self.settings.onboarded = false;
            self.event(EventKind::Error, error.to_string());
            return false;
        }
        self.start_selected_services();
        true
    }

    pub fn issue_report(&self) -> String {
        let tests = self.diagnostics.results.iter().map(|score| {
            format!(
                "{}: {}/{}; {} ms; failed={:?}; error={:?}",
                score.file_name, score.passed, score.total, score.avg_ms,
                score.failed, score.error,
            )
        }).collect::<Vec<_>>().join("\n");
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
                self.settings_status = tr("Отчёт сохранён. Проверьте текст перед публикацией", "Report saved. Review it before publishing").into();
                Some(path)
            }
            Err(error) => {
                self.event(EventKind::Error, error.to_string());
                None
            }
        }
    }
}
