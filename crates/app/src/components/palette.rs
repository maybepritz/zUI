//! Графитовая палитра zUI: одна нейтральная шкала на фон, поверхности, границы и текст.
//!
//! Тема GPUI Kit даёт чисто чёрный фон и одинаковые поверхности. Здесь её цвета
//! заменяются ступенями одной шкалы: боковая панель глубже страницы, карточка
//! светлее страницы, активная строка светлее карточки. Цвет остаётся только у
//! состояний и у графика.
use gpui_kit::{component::*, *};
use std::sync::atomic::{AtomicBool, Ordering};

/// Тёмная ли сейчас палитра. Нужна компонентам, которые строятся без `App`.
static DARK: AtomicBool = AtomicBool::new(true);

pub struct Ramp {
    /// Самая глубокая поверхность: боковая панель.
    pub deep: u32,
    pub page: u32,
    pub card: u32,
    /// Активная строка, поле ввода, выбранный вариант.
    pub raised: u32,
    pub text: u32,
    pub secondary: u32,
    /// Только подписи и служебные значения.
    pub muted: u32,
}

const DARK_RAMP: Ramp = Ramp {
    deep: 0x0f0f10,
    page: 0x141415,
    card: 0x19191b,
    raised: 0x232325,
    text: 0xededed,
    secondary: 0xa1a1a6,
    muted: 0x85858b,
};

const LIGHT_RAMP: Ramp = Ramp {
    deep: 0xf4f4f5,
    page: 0xfafafa,
    card: 0xffffff,
    raised: 0xf4f4f5,
    text: 0x171717,
    secondary: 0x525252,
    muted: 0x737373,
};

pub fn ramp() -> &'static Ramp {
    if DARK.load(Ordering::Relaxed) {
        &DARK_RAMP
    } else {
        &LIGHT_RAMP
    }
}

pub fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// Тонкая граница: цвет текста на 8%, одинаково тихая на любой ступени.
pub fn hairline() -> Hsla {
    color(ramp().text).opacity(0.08)
}

/// Граница выбранного элемента: заметнее обычной, но без цвета.
pub fn hairline_strong() -> Hsla {
    color(ramp().text).opacity(0.28)
}

/// Подписи и служебные значения; основной и вторичный текст берутся из темы.
pub fn muted() -> Hsla {
    color(ramp().muted)
}

/// Накладывает палитру на тему. Вызывать после каждого `Theme::change`: он возвращает цвета темы.
pub fn apply(cx: &mut App) {
    let dark = cx.theme().mode.is_dark();
    DARK.store(dark, Ordering::Relaxed);
    let ramp = ramp();
    let (deep, page, card, raised) = (
        color(ramp.deep),
        color(ramp.page),
        color(ramp.card),
        color(ramp.raised),
    );
    let (text, secondary) = (color(ramp.text), color(ramp.secondary));
    let line = hairline();
    let theme = Theme::global_mut(cx);
    theme.background = page;
    theme.foreground = text;
    theme.muted = raised;
    theme.muted_foreground = secondary;
    theme.border = line;
    theme.input = line;
    theme.popover = card;
    theme.popover_foreground = text;
    theme.accent = raised;
    theme.accent_foreground = text;
    theme.secondary = raised;
    theme.secondary_hover = raised;
    theme.secondary_active = raised;
    theme.secondary_foreground = text;
    theme.sidebar = deep;
    theme.sidebar_foreground = secondary;
    theme.sidebar_border = line;
    theme.sidebar_accent = raised;
    theme.sidebar_accent_foreground = text;
    // Главная кнопка — цвет текста шкалы: почти белая на тёмном, почти чёрная на светлом.
    theme.primary = text;
    theme.primary_hover = text.opacity(0.9);
    theme.primary_active = text.opacity(0.8);
    theme.primary_foreground = page;
    theme.button_primary = text;
    theme.button_primary_hover = text.opacity(0.9);
    theme.button_primary_active = text.opacity(0.8);
    theme.button_primary_foreground = page;
    theme.radius = px(10.);
    theme.radius_lg = px(16.);
}

/// Возвращает палитру, если тему сменили в обход `apply` (например, сама библиотека
/// при смене темы Windows). Сообщает, пришлось ли что-то менять.
pub fn ensure(cx: &mut App) -> bool {
    let dark = cx.theme().mode.is_dark();
    let expected = if dark { &DARK_RAMP } else { &LIGHT_RAMP };
    if cx.theme().background == color(expected.page) && DARK.load(Ordering::Relaxed) == dark {
        return false;
    }
    apply(cx);
    true
}
