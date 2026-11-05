//! Состояние и сценарии приложения без GPUI Context или Window.
mod background;
mod events;
mod launch;
mod lifecycle;
mod preparation;
mod reports;
mod settings;
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
pub(crate) use preparation::PreparationStage;
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
}

impl AppState {
    /// Папка данных zUI: настройки, статистика и история лежат рядом с runtime.
    pub fn data_dir(&self) -> PathBuf {
        self.runtime_dir
            .parent()
            .unwrap_or(&self.runtime_dir)
            .to_path_buf()
    }
}
