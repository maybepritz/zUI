//! Muted status colors and the service status dot shared by every screen.
use gpui_kit::{component::*, *};

/// Working service, applied setting or a fully passed test.
pub fn ok(cx: &App) -> Hsla {
    tone(142.0, cx)
}

/// Failure that needs the user's attention.
pub fn error(cx: &App) -> Hsla {
    tone(4.0, cx)
}

/// Partial result or a limitation, such as missing administrator rights.
pub fn warning(cx: &App) -> Hsla {
    tone(36.0, cx)
}

// One saturation for every status hue, lighter on dark backgrounds so it stays readable.
fn tone(hue: f32, cx: &App) -> Hsla {
    let lightness = if cx.theme().mode.is_dark() {
        0.62
    } else {
        0.38
    };
    hsla(hue / 360.0, 0.48, lightness, 1.0)
}

/// Small dot in front of a service state; the color carries the state.
pub fn status_dot(color: Hsla) -> Div {
    div().flex_shrink_0().size_2().rounded_full().bg(color)
}
