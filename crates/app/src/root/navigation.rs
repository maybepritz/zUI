//! Вкладки и их подписи.
use crate::i18n::tr;
use gpui_kit::assets::IconName as LucideIcon;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Strategies,
    Dns,
    Telegram,
    Statistics,
    Logs,
    Settings,
}

impl Tab {
    pub(super) fn icon(self) -> LucideIcon {
        match self {
            Self::Home => LucideIcon::House,
            Self::Strategies => LucideIcon::SlidersHorizontal,
            Self::Dns => LucideIcon::Network,
            Self::Telegram => LucideIcon::Send,
            Self::Statistics => LucideIcon::ChartNoAxesColumn,
            Self::Logs => LucideIcon::ScrollText,
            Self::Settings => LucideIcon::Settings,
        }
    }

    /// Вкладки для всех.
    pub(super) const MAIN: [Tab; 4] = [Tab::Home, Tab::Telegram, Tab::Statistics, Tab::Settings];

    /// Ручная настройка обхода: отдельная группа «Для опытных».
    pub(super) const ADVANCED: [Tab; 3] = [Tab::Strategies, Tab::Dns, Tab::Logs];

    pub(super) fn title(self) -> String {
        match self {
            Tab::Home => tr("ui-home"),
            Tab::Strategies => tr("ui-strategies"),
            Tab::Dns => "DNS".into(),
            Tab::Telegram => "Telegram".into(),
            Tab::Statistics => tr("ui-statistics"),
            Tab::Logs => tr("ui-logs"),
            Tab::Settings => tr("ui-settings"),
        }
    }
}
