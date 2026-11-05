//! Рамка-контейнер для экранов, которым нужна карточка без заголовка раздела.
use gpui_kit::{component::*, *};

/// Одна рамка для всех карточек: везде одинаковые отступы, скругление и тонкая граница.
pub fn card(cx: &App) -> Div {
    div()
        .v_flex()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
}
