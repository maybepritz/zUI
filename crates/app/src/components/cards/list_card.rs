//! Общая карточка с рамкой: содержимое слева, действия справа.
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
use gpui_kit::{component::*, *};

/// Собирает содержимое и действия вызывающего кода, не навязывая размеры частей.
/// Экран сам решает, растягивать ли содержимое и можно ли сжимать действия.
pub fn list_card(left: impl IntoElement, right: impl IntoElement) -> AnyElement {
    GroupBox::new()
        .outline()
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
