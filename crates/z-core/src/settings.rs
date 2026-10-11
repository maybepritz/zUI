use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use crate::preferences::{Automation, Language, Notifications, StartupServices, ThemePreference};

/// Only settings implemented by the UI are persisted here.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub selected_strategy: String,
    pub dark_theme: bool,
    pub close_to_tray: bool,
    pub onboarded: bool,
    pub autostart: bool,
    pub language: Language,
    pub theme: ThemePreference,
    pub startup: StartupServices,
    pub notifications: Notifications,
    pub automation: Automation,
    pub last_seen_version: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_strategy: String::new(),
            dark_theme: false,
            close_to_tray: true,
            onboarded: false,
            autostart: false,
            language: Language::default(),
            theme: ThemePreference::default(),
            startup: StartupServices::default(),
            notifications: Notifications::default(),
            automation: Automation::default(),
            last_seen_version: String::new(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(text) => Self::from_json(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        save_json(path, self)
    }

    fn from_json(text: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
            .context("Не удалось прочитать настройки")?;
        let legacy = value.get("onboarded").is_none();
        let legacy_theme = value.get("theme").is_none() && value.get("dark_theme").is_some();
        let mut settings: Self = serde_json::from_value(value)
            .context("Не удалось прочитать настройки")?;
        if legacy {
            settings.onboarded = true;
        }
        if legacy_theme {
            settings.theme = if settings.dark_theme {
                ThemePreference::Dark
            } else {
                ThemePreference::Light
            };
        }
        Ok(settings)
    }
}

/// Write beside the destination, then replace it so interrupted writes cannot
/// truncate the last usable settings / DNS recovery snapshot.
pub(crate) fn save_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("json.tmp");
    let mut file = fs::File::create(&temp)?;
    use std::io::Write;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp, path).context("Не удалось сохранить настройки")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_settings_enable_tray_without_losing_preferences() {
        let settings: Settings =
            serde_json::from_str(r#"{"selected_strategy":"general.bat","dark_theme":true}"#)
                .unwrap();
        assert!(settings.close_to_tray);
        assert!(settings.dark_theme);
        assert_eq!(settings.selected_strategy, "general.bat");
    }

    #[test]
    fn explicit_exit_preference_survives_serialization() {
        let settings = Settings {
            close_to_tray: false,
            ..Settings::default()
        };
        let saved = serde_json::to_string(&settings).unwrap();
        let loaded: Settings = serde_json::from_str(&saved).unwrap();
        assert!(!loaded.close_to_tray);
    }

    #[test]
    fn migration_preserves_theme_and_does_not_repeat_setup() {
        let legacy = Settings::from_json(r#"{"dark_theme":true}"#).unwrap();
        assert!(legacy.onboarded);
        assert_eq!(legacy.theme, ThemePreference::Dark);
        let current = Settings::from_json(
            r#"{"dark_theme":true,"theme":"system","onboarded":false}"#,
        ).unwrap();
        assert!(!current.onboarded);
        assert_eq!(current.theme, ThemePreference::System);
        assert!(!current.automation.auto_switch);
        assert!(!current.automation.daily_retest);
    }
}
