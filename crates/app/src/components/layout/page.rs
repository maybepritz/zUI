/*
 * page.rs
 * zui
 *
 * Created by maybepritz on 2026-10-12.
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

/// Scroll the whole tab area, including the gutters around its centered content.
/// Each calling view retains its own scroll position.
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

/// Keep the heading fixed while scrolling the body over the full tab width.
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

pub fn page_fixed(content: Div) -> AnyElement {
    div()
        .v_flex()
        .size_full()
        .p_6()
        .child(centered(content.flex_1().min_h_0()))
        .into_any_element()
}
