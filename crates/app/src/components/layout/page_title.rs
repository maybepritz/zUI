//! Заголовок страницы: крупная лёгкая строка, приглушённое уточнение и пояснение под ней.
use crate::components::palette;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{component::*, *};

/// `dim` продолжает заголовок приглушённым цветом: число, период или текущее значение.
pub fn page_title(
    title: impl Into<SharedString>,
    dim: Option<SharedString>,
    subtitle: Option<SharedString>,
    cx: &App,
) -> Div {
    div()
        .v_flex()
        .flex_1()
        .min_w_0()
        .gap_2()
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_baseline()
                .gap_x_3()
                .text_size(px(30.))
                .line_height(relative(1.15))
                .child(title.into())
                .when_some(dim, |row, dim| {
                    row.child(div().text_color(palette::muted()).child(dim))
                }),
        )
        .when_some(subtitle, |column, subtitle| {
            column.child(
                div()
                    .max_w(px(560.))
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(subtitle),
            )
        })
}
