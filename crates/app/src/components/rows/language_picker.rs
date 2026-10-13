//! Language choices discovered from catalog metadata rather than a Rust enum.
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
            button.primary()
        } else {
            button.outline()
        };
        row = row.child(button.on_click(cx.listener(move |this, _, window, cx| {
            this.on_language(language.clone(), window, cx);
        })));
    }
    row
}
