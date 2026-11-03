//! Реакции GPUI на действия пользователя: dns.
use super::super::RootView;
use gpui_kit::*;

impl RootView {
    pub(crate) fn on_apply_dns(&mut self, key: &str, cx: &mut Context<Self>) {
        self.state.dns.apply(key);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_restore_dns(&mut self, cx: &mut Context<Self>) {
        self.state.dns.restore();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_reset_dns(&mut self, cx: &mut Context<Self>) {
        self.state.dns.reset();
        self.refresh_and_notify(cx);
    }
}
