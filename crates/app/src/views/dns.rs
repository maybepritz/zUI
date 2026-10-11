use crate::i18n::tr;
use gpui_kit::component::button::*;
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
use gpui_kit::component::label::Label;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use z_core::dns::presets;

use crate::{root::RootView, state::AppState};

pub fn render(state: &mut AppState, cx: &mut Context<RootView>) -> AnyElement {
    let active = state.dns.manager.is_active();
    let active_key = state.dns.manager.state().preset.clone();
    let status = state.dns.status.clone();
    let muted = cx.theme().muted_foreground;
    let busy = state.dns.busy();
    let doh_active = state.dns.manager.doh_active();

    let cards = presets()
        .into_iter()
        .enumerate()
        .map(|(ix, preset)| {
            let is_current = active && active_key == preset.key;
            let has_doh = !preset.doh.is_empty();
            let addrs = preset.ipv4.join(", ");
            let key = preset.key.clone();

            GroupBox::new()
                .outline()
                .gap_2()
                .child(
                    div()
                        .h_flex()
                        .w_full()
                        .gap_3()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(
                                    div()
                                        .h_flex()
                                        .flex_wrap()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Label::new(preset.name.clone())
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_family("Consolas")
                                                .text_color(muted)
                                                .child(addrs),
                                        )
                                        .when(has_doh, |this| {
                                            this.child(
                                                div().text_xs().text_color(muted).child("DoH"),
                                            )
                                        })
                                        .when(is_current, |this| {
                                            this.child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(gpui_kit::hsla(
                                                        142.0 / 360.0,
                                                        0.76,
                                                        0.45,
                                                        1.0,
                                                    ))
                                                    .child(if doh_active {
                                                        tr("✓ Применён · DoH зарегистрирован", "Applied · DoH registered")
                                                    } else {
                                                        tr("✓ Применён", "Applied")
                                                    }),
                                            )
                                        }),
                                )
                                .child(
                                    Label::new(preset.description.clone())
                                        .text_xs()
                                        .text_color(muted),
                                ),
                        )
                        .child(
                            div().flex_shrink_0().child(
                                Button::new(format!("apply-dns-{ix}"))
                                    .disabled(busy)
                                    .outline()
                                    .small()
                                    .label(if is_current {
                                        tr("Переприменить", "Reapply")
                                    } else {
                                        tr("Применить", "Apply")
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.state.dns.apply(&key);
                                        cx.notify();
                                    })),
                            ),
                        ),
                )
                .into_any_element()
        })
        .collect::<Vec<_>>();

    let is_admin = z_core::dns::is_elevated();

    div()
        .v_flex()
        .size_full()
        .gap_4()
        .overflow_y_scrollbar()
        .child(
            div()
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
                                        Label::new(tr("⚠ Запущено без прав Администратора", "Running without administrator privileges"))
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(gpui_kit::hsla(38.0 / 360.0, 0.92, 0.50, 1.0)),
                                    )
                                    .child(
                                        Label::new(tr("Для применения DNS и шифрования DoH требуются права Администратора. Перезапустите zapret-ui от имени Администратора.", "DNS and DoH changes require administrator privileges. Use Restart as administrator in Settings."))
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
                        .child(Label::new(tr("DNS-серверы", "DNS servers")).text_lg().font_weight(FontWeight::BOLD))
                        .child(
                            Label::new(
                                tr("Публичные DNS помогают, только если блокировка сделана подменой DNS-ответов. Прокси-DNS (XBox DNS, Comss One) перенаправляют трафик ряда сервисов через сторонние серверы: они видят, какие домены вы запрашиваете.", "Public DNS helps with DNS-based blocking. Proxy DNS (XBox DNS, Comss One) routes selected services through third-party servers that can see the domains you request."),
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
                                .label(tr("Вернуть как было", "Restore previous settings"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.state.dns.restore();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("reset-dns-btn")
                                .disabled(busy)
                                .ghost()
                                .small()
                                .label(tr("Сбросить на DHCP (Авто)", "Reset to DHCP (automatic)"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.state.dns.reset();
                                    cx.notify();
                                })),
                        ),
                )
                .when(!status.is_empty(), |this| {
                    this.child(
                        Label::new(status.clone())
                            .text_xs()
                            .text_color(gpui_kit::hsla(0.0, 0.7, 0.5, 1.0)),
                    )
                }),
        )
        .children(cards)
        .into_any_element()
}
