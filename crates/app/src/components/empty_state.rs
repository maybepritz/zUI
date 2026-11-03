//! One empty state for every list: an icon, a short title and what to do next.
use gpui_kit::{
    assets::IconName,
    component::{
        empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
        *,
    },
    *,
};

/// The caller adds an action with `.content(...)` when there is one.
pub fn empty_state(
    icon: IconName,
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> Empty {
    Empty::new().py_8().header(
        EmptyHeader::new()
            .media(
                EmptyMedia::new()
                    .with_variant(EmptyMediaVariant::Icon)
                    .child(Icon::new(icon).size_4()),
            )
            .title(EmptyTitle::new().child(title.into()))
            .description(EmptyDescription::new().child(detail.into())),
    )
}
