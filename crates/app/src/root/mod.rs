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
}
