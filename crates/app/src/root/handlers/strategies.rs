//! Реакции GPUI на действия пользователя: strategies.
use super::super::RootView;
use gpui_kit::component::input::InputState;
use gpui_kit::*;

impl RootView {
    pub(crate) fn on_strategy_search_changed(
        &mut self,
        input: &Entity<InputState>,
        cx: &mut Context<Self>,
    ) {
        self.strategy_filters.query = input.read(cx).value().to_string();
        cx.notify();
    }

    pub(crate) fn on_favorites_filter_changed(
        &mut self,
        favorites_only: bool,
        cx: &mut Context<Self>,
    ) {
        self.strategy_filters.favorites_only = favorites_only;
        cx.notify();
    }

    pub(crate) fn on_reset_strategy_filters(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Programmatic set_value does not emit Change, so reset the query explicitly.
        self.strategy_filters.query.clear();
        self.strategy_filters.favorites_only = false;
        self.strategy_filters
            .input
            .update(cx, |input, cx| input.set_value("", window, cx));
        cx.notify();
    }

    pub(crate) fn on_toggle_favorite(&mut self, file_name: &str, cx: &mut Context<Self>) {
        self.state.toggle_favorite(file_name);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_start_test(&mut self, cx: &mut Context<Self>) {
        self.state.start_test(false);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_cancel_test(&mut self, cx: &mut Context<Self>) {
        self.state.diagnostics.cancel();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_apply_best_strategy(&mut self, cx: &mut Context<Self>) {
        self.state.apply_best_strategy();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_select_strategy(&mut self, index: usize, cx: &mut Context<Self>) {
        self.state.select_strategy(index);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_install_zapret(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.zapret.install();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_toggle_zapret(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.zapret.toggle();
        self.refresh_and_notify(cx);
    }
}
