//! Сохранение настроек и обработка ошибок записи.

use super::AppState;
use crate::notifications::EventKind;
use z_core::preferences::{Language, ThemePreference};

/// Persisted boolean preferences shared by settings and first-run setup.
#[derive(Clone, Copy)]
pub(crate) enum Preference {
    StartupZapret,
    StartupTelegram,
    CloseToTray,
    RememberCloseAction,
    Watchdog,
    AutoSwitch,
    DailyRetest,
    NotifyErrors,
    NotifyStrategy,
    NotifyUpdates,
}

impl AppState {
    /// Change a preference using the same save/error path as the original UI.
    pub(crate) fn set_preference(&mut self, preference: Preference, value: bool) {
        if self.profile_busy()
            && matches!(
                preference,
                Preference::StartupZapret
                    | Preference::StartupTelegram
                    | Preference::Watchdog
                    | Preference::AutoSwitch
                    | Preference::DailyRetest
            )
        {
            return;
        }
        match preference {
            Preference::StartupZapret => self.settings.startup.zapret = value,
            Preference::StartupTelegram => self.settings.startup.telegram = value,
            Preference::CloseToTray => self.settings.close_to_tray = value,
            Preference::RememberCloseAction => self.settings.remember_close_action = value,
            Preference::Watchdog => self.settings.automation.watchdog = value,
            Preference::AutoSwitch => self.settings.automation.auto_switch = value,
            Preference::DailyRetest => self.settings.automation.daily_retest = value,
            Preference::NotifyErrors => self.settings.notifications.errors = value,
            Preference::NotifyStrategy => self.settings.notifications.strategy = value,
            Preference::NotifyUpdates => self.settings.notifications.updates = value,
        }
        self.save_settings();
    }

    pub(crate) fn set_language(&mut self, language: Language) {
        self.settings.language = language;
    }

    pub(crate) fn set_theme(&mut self, preference: ThemePreference, dark: bool) {
        self.settings.theme = preference;
        self.settings.dark_theme = dark;
    }

    pub(crate) fn reopen_onboarding(&mut self) {
        self.settings.onboarded = false;
    }

    /// Keep the changelog visible when saving the acknowledged version fails.
    pub(crate) fn acknowledge_version(&mut self, version: &str) -> bool {
        self.settings.last_seen_version = version.into();
        self.save_settings()
    }

    pub fn save_settings(&mut self) -> bool {
        match self.settings.save(&self.data_dir().join("settings.json")) {
            Ok(()) => {
                self.settings_status = crate::i18n::text("ui-settings-saved").into();
                true
            }
            Err(error) => {
                self.event(
                    EventKind::Error,
                    crate::i18n::tr_args(
                        "settings-save-error",
                        &[("error", format!("{error:#}").into())],
                    ),
                );
                false
            }
        }
    }
}
