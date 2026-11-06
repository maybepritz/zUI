use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Сохраняемый код BCP-47; для нового перевода не нужен новый вариант перечисления в Rust.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Language(Cow<'static, str>);

impl Language {
    #[allow(non_upper_case_globals)]
    pub const Ru: Self = Self(Cow::Borrowed("ru"));

    pub fn from_code(code: &str) -> Self {
        Self(Cow::Owned(code.to_owned()))
    }
    pub fn code(&self) -> &str {
        &self.0
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::Ru
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct StartupServices {
    pub zapret: bool,
    pub telegram: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Notifications {
    pub errors: bool,
    pub strategy: bool,
    pub updates: bool,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            errors: true,
            strategy: true,
            updates: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct Automation {
    pub watchdog: bool,
    pub auto_switch: bool,
    pub daily_retest: bool,
    pub services: Vec<String>,
}

impl Default for Automation {
    fn default() -> Self {
        Self {
            watchdog: false,
            auto_switch: false,
            daily_retest: false,
            services: crate::services::DEFAULT_IDS.map(str::to_owned).to_vec(),
        }
    }
}
