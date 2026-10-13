//! Журнал, системные уведомления и статус настроек.

use super::AppState;
use crate::notifications::{self, EventKind};

impl AppState {
    pub fn event(&mut self, kind: EventKind, message: String) {
        self.log.add(&message);
        notifications::show(
            &self.settings.notifications,
            kind,
            self.background.hidden,
            &message,
        );
        self.settings_status = message.into();
    }
}
