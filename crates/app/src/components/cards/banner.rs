use crate::i18n::tr;
use crate::root::RootView;
use gpui_kit::component::{
    button::*,
    group_box::{GroupBox, GroupBoxVariants},
    label::Label,
    *,
};
use gpui_kit::*;

pub fn banner(text: impl Into<SharedString>, actions: impl IntoElement) -> AnyElement {
    GroupBox::new()
        .outline()
        .child(
            div()
                .h_flex()
                .w_full()
                .justify_between()
                .items_center()
                .child(Label::new(text.into()).text_sm())
                .child(actions),
        )
        .into_any_element()
}
