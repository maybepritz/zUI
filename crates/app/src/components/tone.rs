//! Цвета состояний и точка состояния сервиса, общие для всех экранов.
//! Это единственный цвет в интерфейсе, кроме линий графика.
use gpui_kit::{component::*, *};

/// Моноширинный шрифт для цифр, адресов и служебных подписей.
pub const MONO: &str = "Consolas";

/// Сервис работает, настройка применена или тест полностью пройден.
pub fn ok(cx: &App) -> Hsla {
    pick(0x4cc38a, 0x16a34a, cx)
}

/// Сбой, который требует внимания пользователя.
pub fn error(cx: &App) -> Hsla {
    pick(0xe5484d, 0xdc2626, cx)
}

/// Частичный результат или ограничение, например нет прав администратора.
pub fn warning(cx: &App) -> Hsla {
    pick(0xe2a336, 0xb45309, cx)
}

// Оттенки приглушены под графитовую шкалу; на светлом фоне темнее, чтобы читалось.
fn pick(dark: u32, light: u32, cx: &App) -> Hsla {
    rgb(if cx.theme().mode.is_dark() {
        dark
    } else {
        light
    })
    .into()
}

/// Маленькая точка перед состоянием сервиса; состояние передаёт цвет.
pub fn status_dot(color: Hsla) -> Div {
    div().flex_shrink_0().size_2().rounded_full().bg(color)
}

/// Цвет линии сервиса на графике: единственное место, где у интерфейса несколько оттенков.
/// Они не совпадают со статусными (зелёный, жёлтый, красный).
pub fn series(index: usize, cx: &App) -> Hsla {
    const DARK: [u32; 4] = [0x5b9df5, 0xa78bfa, 0xf472b6, 0x5ec8c0];
    const LIGHT: [u32; 4] = [0x2563eb, 0x7c3aed, 0xdb2777, 0x0d9488];
    pick(DARK[index % DARK.len()], LIGHT[index % LIGHT.len()], cx)
}

/// Цвет линии сервиса: фирменный, чтобы YouTube узнавался красным, а Discord — сине-фиолетовым.
/// У остальных сервисов — оттенок из общего набора по порядку в списке.
/// Красный YouTube живёт только на графике и в метке плитки; ошибку показывают точка и текст.
pub fn service_color(id: &str, index: usize, cx: &App) -> Hsla {
    match id {
        "youtube" => pick(0xff4e45, 0xe62117, cx),
        "discord" => pick(0x7c86ff, 0x5865f2, cx),
        "instagram" => pick(0xf0609a, 0xd62976, cx),
        "x" => pick(0xc8c8cc, 0x3f3f46, cx),
        "telegram" => pick(0x4cc2ff, 0x229ed9, cx),
        _ => series(index, cx),
    }
}

/// Яркость живой точки от 0.6 до 1: она мерцает, пока обход работает.
pub fn pulse() -> f32 {
    use std::{sync::OnceLock, time::Instant};
    static START: OnceLock<Instant> = OnceLock::new();
    let seconds = START.get_or_init(Instant::now).elapsed().as_secs_f32();
    0.6 + 0.4 * (seconds * 2.6).sin().abs()
}
