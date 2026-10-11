use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    Ru,
    En,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
            updates: true
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
            services: ["chatgpt", "claude", "gemini"].map(str::to_owned).to_vec(),
        }
    }
}
