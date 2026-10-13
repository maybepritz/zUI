//! Состояние и сценарии приложения без GPUI Context или Window.
mod background;
mod events;
mod lifecycle;
mod profiles;
mod reports;
mod settings;
mod startup;
mod strategies;
mod testing;
mod updates;

use crate::i18n::Text;
use std::path::PathBuf;
use z_core::{log::Log, settings::Settings};

use crate::controllers::{
    diagnostics::DiagnosticsController, dns::DnsController, telegram::TelegramController,
    update::UpdateController, zapret::ZapretController,
};

pub(crate) use background::Background;
pub(crate) use profiles::{ProfileError, ProfileRuntime};
pub(crate) use settings::Preference;

pub(crate) struct AppState {
    pub(crate) log: Log,
    pub(crate) zapret: ZapretController,
    pub(crate) dns: DnsController,
    pub(crate) telegram: TelegramController,
    pub(crate) update: UpdateController,
    pub(crate) settings: Settings,
    pub(crate) settings_status: Text,
    pub(crate) runtime_dir: PathBuf,
    pub(crate) diagnostics: DiagnosticsController,
    pub(crate) background: Background,
    pub(crate) profiles: ProfileRuntime,
}
