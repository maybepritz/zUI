//! Владелец GPUI-окна: состояние, жизненный цикл и обработчики событий.
mod handlers;
mod lifecycle;
mod navigation;
mod render;
mod sidebar;
pub(crate) mod strategy_filters;
pub(crate) mod view_data;

use crate::{state::AppState, tray::Tray};
pub(crate) use handlers::onboarding::{OnboardingStep, STEPS};
pub(crate) use navigation::Tab;

pub(crate) struct RootView {
    state: AppState,
    tab: Tab,
    tray: Option<Tray>,
    view_data: view_data::ViewData,
    strategy_filters: strategy_filters::StrategyFilters,
    onboarding_step: OnboardingStep,
    show_whats_new: bool,
    close_prompt_open: bool,
    /// Окно Windows: заголовок перекрашивается вместе с темой.
    hwnd: Option<isize>,
    /// Раскрыт ли блок «Дополнительно» в настройках; между запусками не сохраняется.
    settings_advanced: bool,
    /// График на главной показывает последний час (иначе сутки); между запусками не сохраняется.
    chart_hour: bool,
    /// Текст плашки о VPN, которую закрыли кнопкой «Понятно». Другой VPN или
    /// другие сервисы через него — другой текст, и плашка появится снова.
    vpn_dismissed: Option<String>,
}
