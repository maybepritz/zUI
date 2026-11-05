//! Одно пустое состояние для всех списков: иконка, короткий заголовок и что делать дальше.
use gpui_kit::{
    assets::IconName,
    component::{
        empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
        *,
    },
    *,
};

/// Действие, если оно есть, вызывающий код добавляет через `.content(...)`.
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
