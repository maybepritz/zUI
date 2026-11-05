/*
 * page.rs
 * zui
 *
 * Создан maybepritz 2026-10-12.
 */
use gpui_kit::{
    component::{scroll::ScrollableElement as _, *},
    *,
};
use std::panic::Location;

const PAGE_MAX_WIDTH: f32 = 720.0;

fn centered(content: Div) -> Div {
    content
        .w_full()
        .max_w(px(PAGE_MAX_WIDTH))
        .mx_auto()
        .flex_shrink_0()
}

/// Прокручивает всю область вкладки, включая поля вокруг содержимого по центру.
/// Каждый экран сохраняет свою позицию прокрутки.
#[track_caller]
pub fn page(content: Div) -> AnyElement {
    div()
        .size_full()
        .min_w_0()
        .min_h_0()
        .overflow_y_scrollbar()
        .id(ElementId::CodeLocation(*Location::caller()))
        .child(div().w_full().p_6().child(centered(content)))
        .into_any_element()
}

/// Заголовок остаётся на месте, а тело прокручивается на всю ширину вкладки.
#[track_caller]
pub fn page_with_header(header: Div, content: Div) -> AnyElement {
    div()
        .v_flex()
        .size_full()
        .min_w_0()
        .min_h_0()
        .child(
            div()
                .w_full()
                .p_6()
                .pb_0()
                .flex_shrink_0()
                .child(centered(header)),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .w_full()
                .overflow_y_scrollbar()
                .id(ElementId::CodeLocation(*Location::caller()))
                .child(div().w_full().p_6().pt_4().child(centered(content))),
        )
        .into_any_element()
}
