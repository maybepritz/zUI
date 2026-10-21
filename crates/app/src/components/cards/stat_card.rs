//! A compact metric tile; state and value formatting belong to the caller.
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
    div()
        .v_flex()
        .gap_3()
        .p_4()
        .flex_1()
        .min_w(px(200.))
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().muted.opacity(0.25))
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(Icon::new(icon).size_4().text_color(muted))
                .child(Label::new(title.into()).text_xs().text_color(muted)),
        )
        .child(div().text_2xl().font_weight(FontWeight::BOLD).child(value))
        .child(Label::new(description).text_xs().text_color(muted))
}
