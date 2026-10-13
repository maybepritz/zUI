//! Экран DNS: текущие настройки, восстановление и список пресетов.
mod preset_card;
use crate::components::page::page;
use crate::i18n::tr;
use crate::root::view_data::ViewData;
use gpui_kit::component::button::*;
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
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
    let busy = state.dns.busy() || state.profile_busy();
    let doh_active = state.dns.manager.doh_active();

    let cards = presets()
        .into_iter()
        .enumerate()
        .map(|(ix, preset)| {
            let is_current = active && active_key == preset.key;
            preset_card::render(ix, preset, is_current, busy, doh_active, muted, cx)
        })
        .collect::<Vec<_>>();

    let is_admin = data.administrator;

    page(
        div()
            .v_flex()
            .gap_4()
            .child(div()
                .v_flex()
                .gap_3()
                .when(!is_admin, |this| {
                    this.child(
                        GroupBox::new()
                            .outline()
                            .child(
                                div()
                                    .v_flex()
                                    .gap_1()
                                    .child(
                                        Label::new(tr("ui-running-without-administrator-privileges"))
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(gpui_kit::hsla(38.0 / 360.0, 0.92, 0.50, 1.0)),
                                    )
                                    .child(
                                        Label::new(tr("ui-dns-and-doh-changes-require-administrator-privileges"))
                                            .text_xs()
                                            .text_color(muted),
                                    ),
                            ),
                    )
                })
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(Label::new(tr("ui-dns-servers")).text_lg().font_weight(FontWeight::BOLD))
                        .child(
                            Label::new(
                                tr("ui-public-dns-helps-with-dns-based-blocking"),
                            )
                            .text_xs()
                            .text_color(muted),
                        ),
                )
                .child(
                    div()
                        .h_flex()
                        .flex_wrap()
                        .gap_2()
                        .child(
                            Button::new("restore-dns-btn")
                                .disabled(busy || !active)
                                .outline()
                                .small()
                                .label(tr("ui-restore-previous-settings"))
                                .on_click(cx.listener(|this, _, _, cx| {
    this.on_restore_dns(cx);
})),
                        )
                        .child(
                            Button::new("reset-dns-btn")
                                .disabled(busy)
                                .ghost()
                                .small()
                                .label(tr("ui-reset-to-dhcp-automatic"))
                                .on_click(cx.listener(|this, _, _, cx| {
    this.on_reset_dns(cx);
})),
                        ),
                )
                .when(!status.is_empty(), |this| {
                    this.child(
                        Label::new(status.clone())
                            .text_xs()
                            .text_color(gpui_kit::hsla(0.0, 0.7, 0.5, 1.0)),
                    )
                }),)
            .children(cards),
    )
}
