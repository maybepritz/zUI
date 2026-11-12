//! Сохранение настроек и обработка ошибок записи.

use super::AppState;
use crate::notifications::EventKind;
use z_core::preferences::{Language, ThemePreference};

/// Сохраняемые флажки, общие для настроек и мастера первого запуска.
#[derive(Clone, Copy)]
pub(crate) enum Preference {
    StartupZapret,
    StartupTelegram,
    AskElevation,
    Watchdog,
    AutoSwitch,
    DailyRetest,
    NotifyErrors,
    NotifyStrategy,
    NotifyUpdates,
}

impl AppState {
    /// Меняет настройку тем же путём сохранения и обработки ошибок, что и основной интерфейс.
    pub(crate) fn set_preference(&mut self, preference: Preference, value: bool) {
        match preference {
            Preference::StartupZapret => self.settings.startup.zapret = value,
            Preference::StartupTelegram => self.settings.startup.telegram = value,
            Preference::AskElevation => self.settings.ask_elevation = value,
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

    /// Если не удалось сохранить просмотренную версию, список изменений остаётся на экране.
    pub(crate) fn acknowledge_version(&mut self, version: &str) -> bool {
        self.settings.last_seen_version = version.into();
        self.save_settings()
    }

    pub fn save_settings(&mut self) -> bool {
        match self.settings.save(&self.data_dir().join("settings.json")) {
            Ok(()) => {
                self.settings_status = crate::i18n::text("ui-settings-saved");
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

/// Что делает крестик окна: в настройках это один выбор вместо двух связанных флажков.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloseBehavior {
    /// Каждый раз показывать диалог «Скрыть в трей / Выйти».
    Ask,
    Tray,
    Exit,
}

impl CloseBehavior {
    pub(crate) fn of(settings: &z_core::settings::Settings) -> Self {
        match (settings.remember_close_action, settings.close_to_tray) {
            (false, _) => Self::Ask,
            (true, true) => Self::Tray,
            (true, false) => Self::Exit,
        }
    }

    /// «Спрашивать» не трогает `close_to_tray`: это вариант, отмеченный в диалоге по умолчанию.
    fn apply(self, settings: &mut z_core::settings::Settings) {
        settings.remember_close_action = self != Self::Ask;
        match self {
            Self::Ask => {}
            Self::Tray => settings.close_to_tray = true,
            Self::Exit => settings.close_to_tray = false,
        }
    }
}

impl AppState {
    pub(crate) fn set_close_behavior(&mut self, behavior: CloseBehavior) {
        behavior.apply(&mut self.settings);
        self.save_settings();
    }

    /// Общий выключатель уведомлений; отдельные виды остаются в «Дополнительно».
    pub(crate) fn set_notifications(&mut self, enabled: bool) {
        let notifications = &mut self.settings.notifications;
        notifications.errors = enabled;
        notifications.strategy = enabled;
        notifications.updates = enabled;
        self.save_settings();
    }
}

#[cfg(test)]
mod tests {
    use super::CloseBehavior;
    use z_core::settings::Settings;

    #[test]
    fn close_behavior_round_trips_through_the_two_saved_flags() {
        for behavior in [CloseBehavior::Ask, CloseBehavior::Tray, CloseBehavior::Exit] {
            let mut settings = Settings::default();
            behavior.apply(&mut settings);
            assert_eq!(CloseBehavior::of(&settings), behavior);
        }
    }

    #[test]
    fn asking_keeps_the_choice_preselected_in_the_dialog() {
        let mut settings = Settings::default();
        CloseBehavior::Exit.apply(&mut settings);
        CloseBehavior::Ask.apply(&mut settings);
        assert!(!settings.remember_close_action);
        assert!(!settings.close_to_tray);
    }
}
