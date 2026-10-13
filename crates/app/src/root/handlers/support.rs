//! Реакции GPUI на действия пользователя: support.
use super::super::RootView;
use crate::i18n::tr;
use gpui_kit::*;
const VERSION: &str = env!("CARGO_PKG_VERSION");

impl RootView {
    pub(crate) fn on_dismiss_update(&mut self, cx: &mut Context<Self>) {
        self.state.update.dismiss();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_install_update(&mut self, cx: &mut Context<Self>) {
        self.state.install_update();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_restart(&mut self, cx: &mut Context<Self>) {
        if self.state.restart_app() {
            cx.defer(|cx| cx.quit());
        }
    }

    pub(crate) fn on_dismiss_zapret_update(&mut self, cx: &mut Context<Self>) {
        self.state.zapret.dismiss_update();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_update_zapret(&mut self, cx: &mut Context<Self>) {
        if self.state.profile_busy() {
            return;
        }
        self.state.zapret.update();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_clear_logs(&mut self, cx: &mut Context<Self>) {
        self.state.log.clear();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_dismiss_whats_new(&mut self, cx: &mut Context<Self>) {
        self.show_whats_new = !self.state.acknowledge_version(VERSION);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_check_update(&mut self, cx: &mut Context<Self>) {
        self.state.update.check();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_show_whats_new(&mut self, cx: &mut Context<Self>) {
        self.show_whats_new = true;
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_export_report(&mut self, cx: &mut Context<Self>) {
        match self.state.export_report() {
            Some(path) => {
                cx.open_with_system(&path);
                self.state.settings_status = tr("ui-report-saved").into();
            }
            None => {
                self.state.settings_status = tr("ui-could-not-save-the-report").into();
            }
        }
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_copy_report(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.state.issue_report()));
        self.state.settings_status = tr("ui-copied-review-before-submitting").into();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_open_runtime(&mut self, cx: &mut Context<Self>) {
        cx.reveal_path(&self.state.data_dir());
    }
    pub(crate) fn on_open_issue(&mut self, cx: &mut Context<Self>) {
        cx.open_url("https://github.com/maybepritz/zui/issues/new");
    }
    pub(crate) fn on_quit(&mut self, cx: &mut Context<Self>) {
        cx.defer(|cx| cx.quit());
    }
}
