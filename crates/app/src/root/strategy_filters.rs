//! Transient search state for the Strategies page; does not change service settings.
use super::RootView;
use crate::i18n::tr;
use gpui_kit::{
    component::input::{InputEvent, InputState},
    *,
};

pub(crate) struct StrategyFilters {
    pub input: Entity<InputState>,
    pub query: String,
    pub favorites_only: bool,
    _subscription: Subscription,
}

impl StrategyFilters {
    pub(super) fn new(window: &mut Window, cx: &mut Context<RootView>) -> Self {
        let input =
            cx.new(|cx| InputState::new(window, cx).placeholder(tr("strategy-search-placeholder")));
        let subscription = cx.subscribe(&input, |this, input, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.on_strategy_search_changed(&input, cx);
            }
        });
        Self {
            input,
            query: String::new(),
            favorites_only: false,
            _subscription: subscription,
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        !self.query.is_empty() || self.favorites_only
    }
}
