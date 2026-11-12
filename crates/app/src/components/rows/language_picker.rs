//! Список языков берётся из метаданных каталогов, а не из перечисления Rust.
use crate::{i18n, root::RootView};
use gpui_kit::{
    component::{button::*, *},
    *,
};
use z_core::preferences::Language;

pub(crate) fn language_picker(current: &Language, prefix: &str, cx: &mut Context<RootView>) -> Div {
    let mut row = div().h_flex().flex_wrap().gap_2();
    let selected = i18n::resolved_language(current);
    for locale in i18n::languages() {
        let language = Language::from_code(&locale.code);
        let button = Button::new(format!("{prefix}-{}", locale.code)).label(locale.name);
        let button = if language.code() == selected {
            // Выбранный вариант — приподнятая поверхность с заметной границей, не главная кнопка.
            button
                .outline()
                .bg(crate::components::palette::color(
                    crate::components::palette::ramp().raised,
                ))
                .border_color(crate::components::palette::hairline_strong())
        } else {
            button.ghost()
        };
        row = row.child(button.on_click(cx.listener(move |this, _, window, cx| {
            this.on_language(language.clone(), window, cx);
        })));
    }
    row
}
