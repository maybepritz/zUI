//! Плашка «нет прав администратора»: причина и кнопка перезапуска рядом с ней.
use super::card::card;
use crate::components::tone::{self, status_dot};
use crate::i18n::tr;
use crate::root::RootView;
use gpui_kit::component::{button::*, *};
use gpui_kit::*;

/// `detail` объясняет, что именно на этом экране не сработает без прав.
pub fn admin_notice(detail: impl Into<SharedString>, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    card(cx)
        .h_flex()
        .items_center()
        .gap_4()
        .child(
            div()
                .v_flex()
                .flex_1()
                .min_w_0()
                .gap_1()
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(status_dot(tone::warning(cx)))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child(tr("ui-running-without-administrator-privileges")),
                        ),
                )
                .child(div().text_xs().text_color(muted).child(detail.into())),
        )
        .child(
            div().flex_shrink_0().child(
                Button::new("admin-notice-elevate")
                    .primary()
                    .small()
                    .label(tr("ui-restart-as-administrator"))
                    .on_click(cx.listener(|this, _, _, cx| this.on_elevate(cx))),
            ),
        )
        .into_any_element()
}
