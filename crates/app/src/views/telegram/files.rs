//! Разметка Telegram-прокси — files.
use crate::components::section::{button_row, section};
use crate::{i18n::tr, root::RootView, root::view_data::ViewData, state::AppState};
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn files_section(
    state: &AppState,
    data: &ViewData,
    running: bool,
    muted: Hsla,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let tg = &state.telegram;
    let has_log = data.telegram_has_log;
    let has_config = data.telegram_has_config;

    section(tr("ui-files-and-diagnostics"))
        .child(
            Label::new(crate::i18n::tr_args(
                "data-directory",
                &[("directory", (tg.proxy.dir.display()).to_string().into())],
            ))
            .text_xs()
            .text_color(muted),
        )
        .child(
            button_row()
                .child(
                    Button::new("tg-log")
                        .outline()
                        .label(tr("ui-open-log"))
                        .disabled(!has_log)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_open_telegram_log(cx);
                        })),
                )
                .child(
                    Button::new("tg-config")
                        .outline()
                        .label(tr("ui-open-configuration"))
                        .disabled(running || !has_config)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_open_telegram_config(cx);
                        })),
                )
                .child(
                    Button::new("tg-releases")
                        .ghost()
                        .label(tr("ui-flowseal-releases"))
                        .on_click(cx.listener(|this, _, _, cx| this.on_telegram_releases(cx))),
                ),
        )
}
