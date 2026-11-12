//! Плитка с показателем: подпись, крупное число и пояснение. Без значков и мини-графиков.
use super::card::card;
use crate::components::{palette, tone};
use gpui_kit::{component::*, *};

pub fn stat_card(
    title: impl Into<SharedString>,
    value: String,
    description: String,
    cx: &App,
) -> Div {
    card(cx)
        .gap_3()
        .flex_1()
        .min_w(px(160.))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(30.))
                .line_height(relative(1.))
                .font_weight(FontWeight::MEDIUM)
                .child(value),
        )
        .child(
            div()
                .text_xs()
                .font_family(tone::MONO)
                .text_color(palette::muted())
                .child(description),
        )
}
