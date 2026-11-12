//! Рамка-контейнер для экранов, которым нужна карточка без заголовка раздела.
use crate::components::palette;
use gpui_kit::{component::*, *};

/// Скругление карточек; элементы внутри получают меньшее.
pub const RADIUS: f32 = 16.;

/// Заливка карточек: ступень шкалы над фоном страницы.
pub fn surface() -> Hsla {
    palette::color(palette::ramp().card)
}

/// Та же заливка и граница для `GroupBox`: разделы и строки списков выглядят как карточки.
pub fn surface_style() -> StyleRefinement {
    StyleRefinement::default()
        .bg(surface())
        .border_color(palette::hairline())
        .rounded(px(RADIUS))
}

/// Одна рамка для всех карточек: везде одинаковые отступы, скругление и тонкая граница.
pub fn card(_cx: &App) -> Div {
    div()
        .v_flex()
        .p_5()
        .rounded(px(RADIUS))
        .border_1()
        .border_color(palette::hairline())
        .bg(surface())
}
