use crate::state::Preference;
use crate::{
    components::{
        hint::hint,
        preference::toggle,
        section::{button_row, section},
    },
    i18n::tr,
    root::RootView,
    state::AppState,
};
use gpui_kit::{
    component::{button::*, label::Label, switch::Switch, *},
    *,
};

pub fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;

    let startup = section(tr("ui-startup"))
        .child(toggle(
            "startup-zapret",
            tr("ui-start-zapret-with-zui"),
            state.settings.startup.zapret,
            cx,
            |this, value, cx| this.on_preference(Preference::StartupZapret, value, cx),
        ))
        .child(toggle(
            "startup-telegram",
            tr("ui-start-telegram-proxy-with-zui"),
            state.settings.startup.telegram,
            cx,
            |this, value, cx| this.on_preference(Preference::StartupTelegram, value, cx),
        ))
        .child(
            div()
                .h_flex()
                .justify_between()
                .items_center()
                .child(Label::new(tr("ui-start-at-windows-sign-in")))
                .child(
                    Switch::new("windows-autostart")
                        .checked(state.settings.autostart)
                        .on_change(cx.listener(|this, checked, _, cx| {
                            this.on_autostart(*checked, cx);
                        })),
                ),
        )
        .child(hint(
            Label::new(tr(
                "ui-autostart-does-not-elevate-privileges-zapret-requires",
            )),
            muted,
        ));

    let app = section(tr("ui-application"))
        .child(toggle(
            "remember-close-action",
            tr("close-remember-setting"),
            state.settings.remember_close_action,
            cx,
            |this, value, cx| this.on_preference(Preference::RememberCloseAction, value, cx),
        ))
        .child(toggle(
            "close-to-tray",
            tr("ui-close-to-tray"),
            state.settings.close_to_tray,
            cx,
            |this, value, cx| this.on_preference(Preference::CloseToTray, value, cx),
        ))
        .child(hint(
            Label::new(tr("ui-hiding-the-window-keeps-services-running-exit")),
            muted,
        ))
        .child(
            button_row()
                .child(
                    Button::new("reopen-onboarding")
                        .outline()
                        .icon(IconName::Settings)
                        .label(tr("ui-open-setup-wizard"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_reopen_onboarding(cx);
                        })),
                )
                .child(
                    Button::new("quit-app")
                        .danger()
                        .label(tr("ui-exit-application"))
                        .on_click(cx.listener(|this, _, _, cx| this.on_quit(cx))),
                ),
        );

    div()
        .v_flex()
        .gap_4()
        .child(startup)
        .child(app)
        .into_any_element()
}
