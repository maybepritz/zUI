//! Разметка Telegram-прокси — connection.
use super::port_hint;
use crate::components::section::{button_row, section};
use crate::{i18n::tr, root::RootView, root::view_data::ViewData, state::AppState};
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn connection_section(
    state: &AppState,
    data: &ViewData,
    running: bool,
    muted: Hsla,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let tg = &state.telegram;
    let installed = data.telegram_installed;
    let busy = tg.installing() || state.profile_busy();
    let ready = tg.proxy.ready;

    let install_label = if tg.installing() {
        tr("ui-downloading")
    } else if installed {
        tr("ui-installed")
    } else {
        tr("ui-install-from-github")
    };

    section(tr("ui-connection"))
        .child(Label::new(data.telegram_status.clone()).text_sm())
        .child(Label::new(tg.message.clone()).text_sm())
        .child(
            button_row()
                .child(
                    Button::new("tg-install")
                        .outline()
                        .label(install_label)
                        .disabled(busy || installed)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_install_telegram(cx);
                        })),
                )
                .child(
                    Button::new("tg-toggle")
                        .primary()
                        .label(if running {
                            tr("ui-stop")
                        } else {
                            tr("ui-start")
                        })
                        .disabled(busy || !installed)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_toggle_telegram(cx);
                        })),
                )
                .child(
                    Button::new("tg-connect")
                        .outline()
                        .label(tr("ui-connect-telegram"))
                        .disabled(!ready)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_connect_telegram(cx);
                        })),
                )
                .child(
                    Button::new("tg-copy")
                        .ghost()
                        .label(tr("ui-copy-link"))
                        .disabled(!ready)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_copy_telegram_link(cx);
                        })),
                ),
        )
        .child(Label::new(port_hint()).text_xs().text_color(muted))
}
