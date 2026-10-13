//! Shared outline card with content on the left and actions on the right.
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
use gpui_kit::{component::*, *};

/// Compose caller-owned content and actions without imposing slot-specific sizing.
/// Screens decide whether content expands or actions must not shrink.
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
