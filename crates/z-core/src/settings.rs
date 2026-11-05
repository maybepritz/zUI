use crate::preferences::{Automation, Language, Notifications, StartupServices, ThemePreference};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Only settings implemented by the UI are persisted here.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub selected_strategy: String,
    pub dark_theme: bool,
    pub close_to_tray: bool,
    pub remember_close_action: bool,
    pub onboarded: bool,
    pub autostart: bool,
    pub language: Language,
    pub theme: ThemePreference,
    pub startup: StartupServices,
    pub notifications: Notifications,
    pub automation: Automation,
    pub last_seen_version: String,
    pub favorite_strategies: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_strategy: String::new(),
            dark_theme: false,
            close_to_tray: true,
            remember_close_action: false,
            onboarded: false,
            autostart: false,
            language: Language::default(),
            theme: ThemePreference::default(),
            startup: StartupServices::default(),
            notifications: Notifications::default(),
            automation: Automation::default(),
            last_seen_version: String::new(),
            favorite_strategies: Vec::new(),
        }
    }
}

/// Why a data file was not used as-is and what was loaded instead.
#[derive(Debug)]
pub struct Recovery {
    pub error: anyhow::Error,
    /// The unreadable file, moved aside so the next save cannot overwrite it.
    pub quarantined: Option<PathBuf>,
    /// The previous readable copy was loaded instead of defaults.
    pub from_backup: bool,
}

/// One log line: the error, where the damaged file went and what was loaded.
impl std::fmt::Display for Recovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let moved = self.quarantined.as_deref().map(Path::display);
        let loaded = if self.from_backup {
            "backup"
        } else {
            "defaults"
        };
        match moved {
            Some(file) => write!(f, "{:#}; moved to {file}; loaded {loaded}", self.error),
            None => write!(
                f,
                "{:#}; could not move the file; loaded {loaded}",
                self.error
            ),
        }
    }
}

impl Settings {
    /// Load settings; an unreadable file is moved aside and the backup is tried
    /// before falling back to defaults.
    pub fn load(path: &Path) -> (Self, Option<Recovery>) {
        load_recovering(path, Self::read)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        save_with_backup(path, self, Self::read)
    }

    /// `Ok(None)` when the file does not exist yet.
    fn read(path: &Path) -> Result<Option<Self>> {
        match fs::read_to_string(path) {
            Ok(text) => Self::from_json(&text).map(Some),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn from_json(text: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
            .context("Не удалось прочитать настройки")?;
        let legacy = value.get("onboarded").is_none();
        let legacy_theme = value.get("theme").is_none() && value.get("dark_theme").is_some();
        let mut settings: Self =
            serde_json::from_value(value).context("Не удалось прочитать настройки")?;
        if legacy {
            settings.onboarded = true;
        }
        // До 0.1.2 по умолчанию проверялись только AI-сервисы с геоблоком.
        if settings.automation.services == ["chatgpt", "claude", "gemini"] {
            settings.automation.services = crate::services::DEFAULT_IDS.map(str::to_owned).to_vec();
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

/// `read` returns `Ok(None)` for a missing file, which loads defaults silently.
pub(crate) fn load_recovering<T: Default>(
    path: &Path,
    read: impl Fn(&Path) -> Result<Option<T>>,
) -> (T, Option<Recovery>) {
    let error = match read(path) {
        Ok(value) => return (value.unwrap_or_default(), None),
        Err(error) => error,
    };
    let moved = path.with_extension(format!("json.corrupt-{}", crate::statistics::now()));
    let quarantined = fs::rename(path, &moved).is_ok().then_some(moved);
    let backup = read(&backup_path(path)).ok().flatten();
    let recovery = Recovery {
        error,
        quarantined,
        from_backup: backup.is_some(),
    };
    (backup.unwrap_or_default(), Some(recovery))
}

pub(crate) fn save_with_backup<T>(
    path: &Path,
    value: &impl Serialize,
    read: impl Fn(&Path) -> Result<Option<T>>,
) -> Result<()> {
    // Keep the last readable version; a damaged file must not replace a good backup.
    // Best effort: a failed backup should not block saving the data itself.
    if matches!(read(path), Ok(Some(_))) {
        let _ = fs::copy(path, backup_path(path));
    }
    save_json(path, value)
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
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

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("zui-settings-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn damaged_file_is_moved_aside_and_previous_save_is_restored() {
        let dir = temp_dir("restore");
        let path = dir.join("settings.json");
        for strategy in ["first.bat", "second.bat"] {
            let settings = Settings {
                selected_strategy: strategy.into(),
                ..Default::default()
            };
            settings.save(&path).unwrap();
        }
        fs::write(&path, b"{\"onboarded\": tru").unwrap();

        let (loaded, recovery) = Settings::load(&path);
        let recovery = recovery.unwrap();
        assert!(recovery.from_backup);
        assert_eq!(loaded.selected_strategy, "first.bat");
        let quarantined = recovery.quarantined.unwrap();
        assert_eq!(fs::read(&quarantined).unwrap(), b"{\"onboarded\": tru");

        // The first save after recovery must not replace the good backup.
        loaded.save(&path).unwrap();
        let backup = Settings::read(&backup_path(&path)).unwrap().unwrap();
        assert_eq!(backup.selected_strategy, "first.bat");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn damaged_file_without_backup_survives_saving_defaults() {
        let dir = temp_dir("defaults");
        let path = dir.join("settings.json");
        fs::write(&path, b"not json").unwrap();

        let (loaded, recovery) = Settings::load(&path);
        let recovery = recovery.unwrap();
        assert!(!recovery.from_backup);
        loaded.save(&path).unwrap();
        assert_eq!(
            fs::read(recovery.quarantined.unwrap()).unwrap(),
            b"not json"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn existing_settings_ask_about_closing() {
        let settings = Settings::from_json(r#"{"onboarded":true,"close_to_tray":false}"#).unwrap();
        assert!(!settings.close_to_tray);
        assert!(!settings.remember_close_action);
    }

    #[test]
    fn both_remembered_close_choices_survive_restart() {
        for hide in [false, true] {
            let settings = Settings {
                close_to_tray: hide,
                remember_close_action: true,
                ..Default::default()
            };
            let loaded = Settings::from_json(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert!(loaded.remember_close_action);
            assert_eq!(loaded.close_to_tray, hide);
        }
    }

    #[test]
    fn settings_saved_with_removed_profiles_still_load() {
        let settings = Settings::from_json(
            r#"{"onboarded":true,"selected_strategy":"general.bat","profiles":[{"id":"old","name":"Home"}],"selected_profile_id":"old"}"#,
        )
        .unwrap();
        assert_eq!(settings.selected_strategy, "general.bat");
    }

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
        let current =
            Settings::from_json(r#"{"dark_theme":true,"theme":"system","onboarded":false}"#)
                .unwrap();
        assert!(!current.onboarded);
        assert_eq!(current.theme, ThemePreference::System);
        assert!(!current.automation.auto_switch);
        assert!(!current.automation.daily_retest);
    }

    #[test]
    fn language_codes_preserve_old_settings_and_allow_community_locales() {
        for code in ["ru", "en", "pt-BR", "zh-Hant"] {
            let settings =
                Settings::from_json(&format!(r#"{{"language":"{code}","onboarded":true}}"#))
                    .unwrap();
            assert_eq!(settings.language.code(), code);
            let loaded: Settings =
                serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert_eq!(loaded.language.code(), code);
        }
    }
}
