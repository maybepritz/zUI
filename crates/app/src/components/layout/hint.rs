use gpui_kit::{component::label::Label, *};

/// Приглушённая мелкая подсказка под настройкой
pub fn hint(text: Label, muted: Hsla) -> Label {
    text.text_sm().text_color(muted)
}
