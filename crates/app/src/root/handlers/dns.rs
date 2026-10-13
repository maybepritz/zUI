//! Реакции GPUI на действия пользователя: dns.
use super::super::RootView;
use gpui_kit::*;

impl RootView {
    pub(crate) fn on_apply_dns(&mut self, key: &str, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.dns.apply(key);
        self.state.settings.selected_profile_id = None;
        self.state.save_settings();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_restore_dns(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.dns.restore();
        self.state.settings.selected_profile_id = None;
        self.state.save_settings();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_reset_dns(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.dns.reset();
        self.state.settings.selected_profile_id = None;
        self.state.save_settings();
        self.refresh_and_notify(cx);
    }
}
