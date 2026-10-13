//! Presentation-only favorite toggle. The caller attaches its event handler.
use gpui_kit::assets::IconName as LucideIcon;
use gpui_kit::{
    component::{button::*, *},
    *,
};

pub(crate) fn favorite_color(cx: &App) -> Hsla {
    if cx.theme().mode.is_dark() {
        hsla(42.0 / 360.0, 0.95, 0.65, 1.0)
    } else {
        hsla(36.0 / 360.0, 0.90, 0.30, 1.0)
    }
}

pub(crate) fn favorite_button(
    id: impl Into<ElementId>,
    favorite: bool,
    tooltip: impl Into<SharedString>,
    cx: &App,
) -> Button {
    let icon = if favorite {
        LucideIcon::StarFill
    } else {
        LucideIcon::Star
    };
    let style = ButtonCustomVariant::new(cx)
        .foreground(if favorite {
            favorite_color(cx)
        } else {
            cx.theme().muted_foreground
        })
        .color(hsla(
            40.0 / 360.0,
            0.85,
            0.55,
            if favorite { 0.12 } else { 0.0 },
        ))
        .hover(hsla(40.0 / 360.0, 0.85, 0.55, 0.18))
        .active(hsla(40.0 / 360.0, 0.85, 0.55, 0.25));

    Button::new(id)
        .small()
        .custom(style)
        .icon(icon)
        .tooltip(tooltip)
        .toggled(favorite)
}
