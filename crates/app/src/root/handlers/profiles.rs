//! Profile dialog ownership and GPUI callbacks.
use super::super::RootView;
use crate::{
    components::modal::form_dialog,
    i18n,
    state::ProfileError,
    views::profiles::form::{ProfileDialog, ProfileForm},
};
use gpui_kit::{
    component::{WindowExt, input::InputState},
    *,
};
use std::cell::Cell;

impl RootView {
    pub(crate) fn on_apply_profile(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Err(error) = self.state.apply_profile(id) {
            self.state.profiles.status = error.text();
        }
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_profile_dialog(
        &mut self,
        kind: ProfileDialog,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = match kind.id() {
            Some(id) => match self.state.settings.profiles.iter().find(|p| p.id == id) {
                Some(profile) => profile.name.clone(),
                None => {
                    self.state.profiles.status = ProfileError::NotFound.text();
                    self.refresh_and_notify(cx);
                    return;
                }
            },
            None => String::new(),
        };
        let form = cx.new(|cx| ProfileForm::new(kind.clone(), name, window, cx));
        let root = cx.entity().downgrade();
        let focus = Cell::new(kind.has_name());
        window.open_dialog(cx, move |dialog, window, cx| {
            if focus.replace(false) {
                let input: Entity<InputState> = form.read(cx).input.clone();
                input.focus_handle(cx).focus(window, cx);
            }
            let submit_form = form.clone();
            let submit_root = root.clone();
            let submit_kind = kind.clone();
            form_dialog(dialog, i18n::tr(kind.title()), i18n::tr(kind.confirm()))
                .child(form.clone())
                .on_ok(move |_, _, cx| {
                    let name = submit_form.read(cx).input.read(cx).value().to_string();
                    let dns = submit_form.read(cx).dns_override();
                    let result = submit_root.update(cx, |this, cx| {
                        let result = match &submit_kind {
                            ProfileDialog::Create => {
                                this.state.save_current_profile(&name, dns).map(|_| ())
                            }
                            ProfileDialog::Rename(id) => this.state.rename_profile(id, &name),
                            ProfileDialog::Update(id) => this.state.update_profile(id, dns),
                            ProfileDialog::Delete(id) => this.state.delete_profile(id),
                        };
                        this.refresh_and_notify(cx);
                        result
                    });
                    match result {
                        Ok(Ok(())) => true,
                        Ok(Err(error)) => {
                            submit_form.update(cx, |form, cx| {
                                form.error = Some(error.text());
                                cx.notify();
                            });
                            false
                        }
                        Err(_) => true,
                    }
                })
        });
    }
}
