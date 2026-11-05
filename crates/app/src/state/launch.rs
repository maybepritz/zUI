//! Запуск сервисов, перезапуск и завершение мастера.

use super::AppState;
use crate::{notifications::EventKind, platform};

impl AppState {
    pub fn restart_app(&mut self) -> bool {
        match self.update.restart() {
            Ok(()) => true,
            Err(error) => {
                self.event(EventKind::Error, error.to_string());
                false
            }
        }
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
}
