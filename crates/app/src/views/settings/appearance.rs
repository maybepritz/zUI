use crate::{
    components::section::{button_row, section},
    i18n::tr,
    root::RootView,
    state::AppState,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};
use z_core::preferences::ThemePreference;

pub fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let mut themes = button_row();
    for (id, name, preference) in [
        ("theme-system", tr("ui-system"), ThemePreference::System),
        ("theme-light", tr("ui-light"), ThemePreference::Light),
        ("theme-dark", tr("ui-dark"), ThemePreference::Dark),
    ] {
        let selected = state.settings.theme == preference;
        let button = Button::new(id).label(name);
        let button = if selected {
            button.primary()
        } else {
            button.outline()
        };
        themes = themes.child(button.on_click(cx.listener(move |this, _, window, cx| {
            this.on_theme(preference, window, cx);
        })));
    }

    let languages = crate::components::rows::language_picker::language_picker(
        &state.settings.language,
        "settings",
        cx,
    );

    let translators = crate::i18n::translators(&state.settings.language).join(", ");
    section(tr("ui-appearance"))
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(Label::new(tr("ui-theme")))
                .child(themes),
        )
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(Label::new(tr("ui-interface-language")))
                .child(languages)
                .when(!translators.is_empty(), |row| {
                    row.child(
                        Label::new(crate::i18n::tr_args(
                            "locale-credit",
                            &[("names", translators.into())],
                        ))
                        .text_xs(),
                    )
                }),
        )
        .into_any_element()
}
