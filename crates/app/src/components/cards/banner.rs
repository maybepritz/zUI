//! Простая плашка: текст слева, действия справа.
use gpui_kit::component::{
    group_box::{GroupBox, GroupBoxVariants},
    *,
};
use gpui_kit::*;

/// Длинный текст (например, ошибка с адресом) переносится и не выталкивает
/// кнопки за край окна: у текста `min_w_0`, у действий запрет сжатия.
pub fn banner(text: impl Into<SharedString>, actions: impl IntoElement) -> AnyElement {
    GroupBox::new()
        .outline()
        .content_style(super::card::surface_style())
        .child(
            div()
                .h_flex()
                .w_full()
                .gap_4()
                .justify_between()
                .items_center()
                .child(div().flex_1().min_w_0().text_sm().child(text.into()))
                .child(div().flex_shrink_0().child(actions)),
        )
        .into_any_element()
}
