//! Реакции GPUI на действия пользователя: navigation.
use super::super::RootView;
use super::super::Tab;
use gpui_kit::*;

impl RootView {
    pub(crate) fn on_show_logs(&mut self, cx: &mut Context<Self>) {
        self.tab = Tab::Logs;
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_navigate(&mut self, tab: Tab, cx: &mut Context<Self>) {
        self.tab = tab;
        self.refresh_and_notify(cx);
    }
}
