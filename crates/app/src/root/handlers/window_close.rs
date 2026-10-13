//! Close-button policy and explicit Hide / Exit actions.
use super::super::RootView;
use crate::{
    components::modal::modal,
    i18n::{self, Text},
    tray::Tray,
    views::overlays::close_dialog::CloseDialog,
};
use gpui_kit::{
    component::{
        Disableable as _, WindowExt,
        button::*,
        dialog::{DialogClose, DialogFooter},
    },
    *,
};

impl RootView {
    pub(crate) fn on_close_requested(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Native close callback must return before rebuilding a dialog layer.
        cx.defer_in(window, |this, window, cx| {
            this.show_close_choice(window, cx)
        });
    }

    fn show_close_choice(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.close_prompt_open {
            return;
        }
        if self.state.settings.remember_close_action {
            let hide = self.state.settings.close_to_tray;
            if self.finish_close_choice(hide, false, cx).is_ok() {
                return;
            }
            // A missing tray must never make a remembered Hide choice trap the user.
            self.state.settings.remember_close_action = false;
            self.state.save_settings();
        }
        self.close_prompt_open = true;
        let can_hide = self.tray.is_some();
        let form = cx.new(|_| CloseDialog {
            error: (!can_hide).then(|| i18n::text("window-close-tray-failed")),
            ..Default::default()
        });
        let root = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, _| {
            let hide_form = form.clone();
            let hide_root = root.clone();
            let exit_form = form.clone();
            let exit_root = root.clone();
            let close_root = root.clone();
            modal(dialog, i18n::tr("close-dialog-title"))
                .child(form.clone())
                .on_ok(|_, _, _| false)
                .on_close(move |_, _, cx| {
                    let _ = close_root.update(cx, |this, _| this.close_prompt_open = false);
                })
                .footer(
                    DialogFooter::new()
                        .child(
                            DialogClose::new()
                                .trigger(|button| button.outline().label(i18n::tr("ui-cancel"))),
                        )
                        .child(
                            Button::new("close-hide")
                                .outline()
                                .label(i18n::tr("close-hide"))
                                .disabled(!can_hide)
                                .on_click(
                                    move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                                        let remember = hide_form.read(cx).remember;
                                        let result = hide_root.update(cx, |this, cx| {
                                            this.finish_close_choice(true, remember, cx)
                                        });
                                        match result {
                                            Ok(Ok(())) => window.close_dialog(cx),
                                            Ok(Err(error)) => hide_form.update(cx, |form, cx| {
                                                form.error = Some(error);
                                                cx.notify();
                                            }),
                                            Err(_) => window.close_dialog(cx),
                                        }
                                    },
                                ),
                        )
                        .child(
                            Button::new("close-exit")
                                .danger()
                                .label(i18n::tr("close-exit"))
                                .on_click(
                                    move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                                        let remember = exit_form.read(cx).remember;
                                        let result = exit_root.update(cx, |this, cx| {
                                            this.finish_close_choice(false, remember, cx)
                                        });
                                        match result {
                                            Ok(Ok(())) => window.close_dialog(cx),
                                            Ok(Err(error)) => exit_form.update(cx, |form, cx| {
                                                form.error = Some(error);
                                                cx.notify();
                                            }),
                                            Err(_) => window.close_dialog(cx),
                                        }
                                    },
                                ),
                        ),
                )
        });
    }

    fn finish_close_choice(
        &mut self,
        hide: bool,
        remember: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), Text> {
        if hide && self.tray.is_none() {
            return Err(i18n::text("window-close-tray-failed"));
        }
        let previous = (
            self.state.settings.close_to_tray,
            self.state.settings.remember_close_action,
        );
        if remember {
            self.state.settings.close_to_tray = hide;
            self.state.settings.remember_close_action = true;
            if !self.state.save_settings() {
                (
                    self.state.settings.close_to_tray,
                    self.state.settings.remember_close_action,
                ) = previous;
                return Err(i18n::text("close-error-save"));
            }
        }
        if hide {
            if !self.tray.as_ref().is_some_and(Tray::hide) {
                if remember {
                    (
                        self.state.settings.close_to_tray,
                        self.state.settings.remember_close_action,
                    ) = previous;
                    self.state.save_settings();
                }
                return Err(i18n::text("window-close-tray-failed"));
            }
            self.state.background.hidden = true;
        } else {
            cx.defer(|cx| cx.quit());
        }
        self.close_prompt_open = false;
        Ok(())
    }
}
