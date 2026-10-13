/*
 * profiles.rs
 * zui
 *
 * Created by maybepritz on 2026-10-13.
 */

use crate::preferences::{Automation, StartupServices};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub strategy: String,

    #[serde(default)]
    pub dns: ProfileDns,

    #[serde(default)]
    pub startup: StartupServices,

    #[serde(default)]
    pub automation: Automation,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "mode", content = "key", rename_all = "snake_case")]
pub enum ProfileDns {
    #[default]
    Keep,
    Automatic,
    Preset(String),
}
