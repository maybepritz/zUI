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

    let startup = section(tr("Запуск", "Startup"))
        .child(toggle(
            "startup-zapret",
            tr("Запускать Zapret вместе с zUI", "Start Zapret with zUI"),
            state.settings.startup.zapret,
            cx,
            |s, value| s.settings.startup.zapret = value,
        ))
        .child(toggle(
            "startup-telegram",
            tr(
                "Запускать Telegram-прокси вместе с zUI",
                "Start Telegram proxy with zUI",
            ),
            state.settings.startup.telegram,
            cx,
            |s, value| s.settings.startup.telegram = value,
        ))
        .child(
            div()
                .h_flex()
                .justify_between()
                .items_center()
                .child(Label::new(tr(
                    "Автозапуск при входе в Windows",
                    "Start at Windows sign-in",
                )))
                .child(
                    Switch::new("windows-autostart")
                        .checked(state.settings.autostart)
                        .on_change(cx.listener(|this, checked, _, cx| {
                            this.state.set_autostart(*checked);
                            cx.notify();
                        })),
                ),
        )
        .child(hint(
            Label::new(tr(
                "Автозапуск не повышает права. Для Zapret понадобится запуск от администратора.",
                "Autostart does not elevate privileges. Zapret requires an administrator launch.",
            )),
            muted,
        ));

    let app = section(tr("Приложение", "Application"))
        .child(toggle(
            "close-to-tray",
            tr("Сворачивать в трей при закрытии", "Close to tray"),
            state.settings.close_to_tray,
            cx,
            |s, value| s.settings.close_to_tray = value,
        ))
        .child(hint(
            Label::new(tr(
                "Скрытое окно сохраняет работу сервисов; «Выйти» останавливает их.",
                "Hiding the window keeps services running; Exit stops them.",
            )),
            muted,
        ))
        .child(
            button_row()
                .child(
                    Button::new("reopen-onboarding")
                        .outline()
                        .icon(IconName::Settings)
                        .label(tr("Открыть мастер настройки", "Open setup wizard"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.settings.onboarded = false;
                            this.onboarding_step = Default::default();
                            this.state.save_settings();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("quit-app")
                        .danger()
                        .label(tr("Выйти из приложения", "Exit application"))
                        .on_click(cx.listener(|_, _, _, cx| cx.defer(|cx| cx.quit()))),
                ),
        );

    div()
        .v_flex()
        .gap_4()
        .child(startup)
        .child(app)
        .into_any_element()
}
