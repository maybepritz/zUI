//! Экран DNS: текущие настройки, восстановление и список пресетов.
mod preset_card;
use crate::components::cards::admin_notice::admin_notice;
use crate::components::page::page;
use crate::components::page_title::page_title;
use crate::components::tone;
use crate::i18n::tr;
use crate::root::view_data::ViewData;
use gpui_kit::component::button::*;
use gpui_kit::component::label::Label;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use z_core::dns::presets;

use crate::{root::RootView, state::AppState};

pub fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let active = state.dns.manager.is_active();
    let active_key = state.dns.manager.state().preset.clone();
    let status = state.dns.status.clone();
    let muted = cx.theme().muted_foreground;
    let error = tone::error(cx);
    let busy = state.dns.busy();
    let doh_active = state.dns.manager.doh_active();
    let is_admin = data.administrator;

    let cards = presets()
        .into_iter()
        .enumerate()
        .map(|(ix, preset)| {
            let is_current = active && active_key == preset.key;
            preset_card::render(ix, preset, is_current, busy, doh_active, muted, cx)
        })
        .collect::<Vec<_>>();

    // Заголовок сразу отвечает на главный вопрос экрана: какой DNS сейчас включён.
    let header = div()
        .h_flex()
        .items_start()
        .gap_4()
        .child(page_title(
            "DNS",
            // Имя включённого пресета продолжает заголовок; без него остаётся просто «DNS».
            active.then(|| state.dns.active_name.to_string().into()),
            Some(tr("ui-public-dns-helps-with-dns-based-blocking").into()),
            cx,
        ))
        .child(
            div().pt_2().flex_shrink_0().child(
                Button::new("restore-dns-btn")
                    .disabled(busy || !active)
                    .outline()
                    .label(tr("ui-restore-previous-settings"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.on_restore_dns(cx);
                    })),
            ),
        );

    page(
        div()
            .v_flex()
            .gap_5()
            .child(header)
            // Причина, по которой кнопки ниже не сработают, стоит прямо над ними.
            .when(!is_admin, |this| {
                this.child(admin_notice(
                    tr("ui-dns-and-doh-changes-require-administrator-privileges"),
                    cx,
                ))
            })
            .when(!status.is_empty(), |this| {
                this.child(Label::new(status.clone()).text_xs().text_color(error))
            })
            .child(div().v_flex().gap_3().children(cards))
            // Аварийный выход стоит отдельно от списка и не спорит с основными действиями.
            .child(
                div().h_flex().child(
                    Button::new("reset-dns-btn")
                        .disabled(busy)
                        .link()
                        .small()
                        .label(tr("ui-reset-to-dhcp-automatic"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_reset_dns(cx);
                        })),
                ),
            ),
    )
}
