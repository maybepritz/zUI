//! Разметка Telegram-прокси.
use crate::i18n::tr;
use crate::root::view_data::ViewData;
use crate::{components::page::page, root::RootView, state::AppState};
use gpui_kit::component::{label::Label, *};
use gpui_kit::*;

mod connection;
mod files;
mod media;
use connection::connection_section;
use files::files_section;
use media::media_section;

fn intro() -> String {
    tr("ui-flowseal-proxy-runs-separately-from-zapret-connect")
}

fn port_hint() -> String {
    tr("ui-the-port-status-confirms-that-the-local")
}

fn media_hint() -> String {
    tr("ui-stop-the-proxy-try-dc-4-only")
}

pub fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let running = data.telegram_running;
    let muted = cx.theme().muted_foreground;

    page(
        div()
            .v_flex()
            .gap_4()
            .child(
                Label::new(tr("telegram-title"))
                    .text_lg()
                    .font_weight(FontWeight::BOLD),
            )
            .child(Label::new(intro()).text_sm().text_color(muted))
            .child(connection_section(state, data, running, muted, cx))
            .child(media_section(state, data, running, cx))
            .child(files_section(state, data, running, muted, cx)),
    )
}
