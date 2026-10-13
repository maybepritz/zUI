//! Реакции GPUI на действия пользователя: telegram.
use super::super::RootView;
use crate::i18n::tr;
use gpui_kit::*;
fn msg_opened() -> String {
    tr("ui-confirm-the-connection-in-telegram-if-no")
}
fn msg_copied() -> String {
    tr("ui-link-copied-open-it-in-telegram-saved")
}

impl RootView {
    pub(crate) fn on_install_telegram(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.telegram.install();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_toggle_telegram(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.telegram.toggle();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_connect_telegram(&mut self, cx: &mut Context<Self>) {
        if let Some(link) = self.state.telegram.link() {
            cx.open_url(&link);
            self.state.telegram.message = msg_opened().into();
        }
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_copy_telegram_link(&mut self, cx: &mut Context<Self>) {
        if let Some(link) = self.state.telegram.link() {
            cx.write_to_clipboard(ClipboardItem::new_string(link));
            self.state.telegram.message = msg_copied().into();
        }
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_fix_telegram_media(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.telegram.fix_media(false);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_clear_telegram_media(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.telegram.fix_media(true);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_open_telegram_log(&mut self, cx: &mut Context<Self>) {
        cx.open_with_system(&self.state.telegram.proxy.log_path());
    }

    pub(crate) fn on_open_telegram_config(&mut self, cx: &mut Context<Self>) {
        cx.open_with_system(&self.state.telegram.proxy.config_path());
    }

    pub(crate) fn on_telegram_releases(&mut self, cx: &mut Context<Self>) {
        cx.open_url(z_core::telegram::RELEASES_URL);
    }
}
