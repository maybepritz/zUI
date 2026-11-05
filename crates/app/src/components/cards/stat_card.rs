//! Компактная плитка с показателем; состояние и форматирование значения задаёт вызывающий код.
use super::card::card;
use gpui_kit::{
    assets::IconName,
    component::{label::Label, *},
    *,
};

pub fn stat_card(
    title: impl Into<SharedString>,
    value: String,
    description: String,
    icon: IconName,
    cx: &App,
) -> Div {
    let muted = cx.theme().muted_foreground;
    card(cx)
        .gap_3()
        .flex_1()
        .min_w(px(200.))
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(Icon::new(icon).size_4().text_color(muted))
                .child(Label::new(title.into()).text_xs().text_color(muted)),
        )
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::MEDIUM)
                .child(value),
        )
        .child(Label::new(description).text_xs().text_color(muted))
}
