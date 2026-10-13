//! Bordered statistic tile; value formatting belongs to the caller.
use crate::components::hint::hint;
use gpui_kit::{
    component::{label::Label, *},
    *,
};

/// Display a titled metric using caller-supplied theme colors and value text.
pub fn stat_card(title: impl Into<SharedString>, value: String, muted: Hsla, border: Hsla) -> Div {
    div()
        .v_flex()
        .gap_1()
        .p_4()
        .flex_1()
        .min_w(px(200.))
        .rounded_lg()
        .border_1()
        .border_color(border)
        .child(hint(Label::new(title.into()), muted))
        .child(div().text_2xl().font_weight(FontWeight::BOLD).child(value))
}
