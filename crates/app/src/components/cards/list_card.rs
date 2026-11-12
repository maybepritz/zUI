//! Общая карточка с рамкой: содержимое слева, действия справа.
use super::card::surface_style;
use crate::components::palette;
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
use gpui_kit::{component::*, *};

/// Собирает содержимое и действия вызывающего кода, не навязывая размеры частей.
/// Экран сам решает, растягивать ли содержимое и можно ли сжимать действия.
pub fn list_card(left: impl IntoElement, right: impl IntoElement) -> AnyElement {
    row(left, right, false)
}

/// Та же карточка для выбранного пункта списка: приподнятая поверхность и заметная
/// граница. Цвета нет: состояние «работает» показывает точка внутри.
pub fn selected_list_card(left: impl IntoElement, right: impl IntoElement) -> AnyElement {
    row(left, right, true)
}

fn row(left: impl IntoElement, right: impl IntoElement, selected: bool) -> AnyElement {
    let style = if selected {
        surface_style()
            .bg(palette::color(palette::ramp().raised))
            .border_color(palette::hairline_strong())
    } else {
        surface_style()
    };
    GroupBox::new()
        .outline()
        .content_style(style)
        .gap_2()
        .child(
            div()
                .h_flex()
                .w_full()
                .gap_3()
                .justify_between()
                .items_center()
                .child(left)
                .child(right),
        )
        .into_any_element()
}
