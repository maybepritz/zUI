//! Карточка DNS-пресета: данные и callback применения.
use crate::components::list_card::list_card;
use crate::{i18n::tr, root::RootView};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn render(
    ix: usize,
    preset: z_core::dns::DnsPreset,
    is_current: bool,
    busy: bool,
    doh_active: bool,
    muted: Hsla,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let has_doh = !preset.doh.is_empty();
    let addrs = preset.ipv4.join(", ");
    let key = preset.key.clone();

    list_card(
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
                        this.child(div().text_xs().text_color(muted).child("DoH"))
                    })
                    .when(is_current, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(gpui_kit::hsla(142.0 / 360.0, 0.76, 0.45, 1.0))
                                .child(if doh_active {
                                    tr("ui-applied-doh-registered")
                                } else {
                                    tr("ui-applied")
                                }),
                        )
                    }),
            )
            .child(
                Label::new(tr(if preset.key == "comss" {
                    "dns-preset-comss-description"
                } else {
                    "dns-preset-public-description"
                }))
                .text_xs()
                .text_color(muted),
            ),
        div().flex_shrink_0().child(
            Button::new(format!("apply-dns-{ix}"))
                .disabled(busy)
                .outline()
                .small()
                .label(if is_current {
                    tr("ui-reapply")
                } else {
                    tr("ui-apply")
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.on_apply_dns(&key, cx);
                })),
        ),
    )
}
