//! Приглушённые цвета статусов и точка состояния сервиса, общие для всех экранов.
use gpui_kit::{component::*, *};

/// Сервис работает, настройка применена или тест полностью пройден.
pub fn ok(cx: &App) -> Hsla {
    tone(142.0, cx)
}

/// Сбой, который требует внимания пользователя.
pub fn error(cx: &App) -> Hsla {
    tone(4.0, cx)
}

/// Частичный результат или ограничение, например нет прав администратора.
pub fn warning(cx: &App) -> Hsla {
    tone(36.0, cx)
}

// Одна насыщенность для всех оттенков статуса; на тёмном фоне светлее, чтобы читалось.
fn tone(hue: f32, cx: &App) -> Hsla {
    let lightness = if cx.theme().mode.is_dark() {
        0.62
    } else {
        0.38
    };
    hsla(hue / 360.0, 0.48, lightness, 1.0)
}

/// Маленькая точка перед состоянием сервиса; состояние передаёт цвет.
pub fn status_dot(color: Hsla) -> Div {
    div().flex_shrink_0().size_2().rounded_full().bg(color)
}
