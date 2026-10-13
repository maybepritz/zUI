//! Разметка Telegram-прокси — media.
use super::media_hint;
use crate::components::section::{button_row, section};
use crate::{i18n::tr, root::RootView, root::view_data::ViewData, state::AppState};
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn media_section(
    state: &AppState,
    data: &ViewData,
    running: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let tg = &state.telegram;
    let blocked = running || tg.installing() || !data.telegram_has_config;

    section(tr("ui-if-photos-or-videos-fail-to-load"))
        .child(Label::new(media_hint()).text_sm())
        .child(
            button_row()
                .child(
                    Button::new("tg-media")
                        .outline()
                        .label(tr("ui-use-dc-4-only"))
                        .disabled(blocked)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_fix_telegram_media(cx);
                        })),
                )
                .child(
                    Button::new("tg-media-clear")
                        .outline()
                        .label(tr("ui-clear-dc-ip"))
                        .disabled(blocked)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_clear_telegram_media(cx);
                        })),
                ),
        )
}
