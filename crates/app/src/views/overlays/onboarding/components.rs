//! Разметка мастера первого запуска — services.
use crate::components::service_row::service_row;
use crate::{components::preference::toggle, state::Preference};
use crate::{i18n::tr, root::RootView, root::view_data::ViewData, state::AppState};
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn services(
    state: &AppState,
    data: &ViewData,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let zapret_busy = state.zapret.installing();
    let tg_busy = state.telegram.installing();

    div()
        .v_flex()
        .gap_4()
        .child(toggle(
            "wizard-zapret",
            tr("ui-start-zapret-with-zui"),
            state.settings.startup.zapret,
            cx,
            |this, value, cx| this.on_preference(Preference::StartupZapret, value, cx),
        ))
        .child(toggle(
            "wizard-telegram",
            tr("ui-start-telegram-proxy-with-zui"),
            state.settings.startup.telegram,
            cx,
            |this, value, cx| this.on_preference(Preference::StartupTelegram, value, cx),
        ))
        .child(service_row(
            Label::new("Zapret"),
            Label::new(state.zapret.status.clone()),
            zapret_busy,
            Button::new("wizard-install-zapret")
                .label(if data.zapret_installed {
                    tr("ui-installed")
                } else {
                    tr("ui-install")
                })
                .disabled(data.zapret_installed || zapret_busy)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.on_install_zapret(cx);
                })),
        ))
        .child(service_row(
            Label::new(tr("ui-telegram-proxy")),
            Label::new(state.telegram.message.clone()),
            tg_busy,
            Button::new("wizard-install-telegram")
                .label(if data.telegram_installed {
                    tr("ui-installed")
                } else {
                    tr("ui-install")
                })
                .disabled(data.telegram_installed || tg_busy)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.on_install_telegram(cx);
                })),
        ))
        .into_any_element()
}
