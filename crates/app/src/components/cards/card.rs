//! Bordered container for screens that need a card without a titled section.
use gpui_kit::{component::*, *};

/// One frame for every card: the same padding, radius and hairline everywhere.
pub fn card(cx: &App) -> Div {
    div()
        .v_flex()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
}
