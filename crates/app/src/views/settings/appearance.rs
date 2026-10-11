use crate::{
    components::section::{button_row, section},
    i18n::{self, tr},
    root::RootView,
    state::AppState,
};
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};
use z_core::preferences::{Language, ThemePreference};

pub fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let mut themes = button_row();
    for (id, name, preference) in [
        (
            "theme-system",
            tr("Как в системе", "System"),
            ThemePreference::System,
        ),
        (
            "theme-light",
            tr("Светлая", "Light"),
            ThemePreference::Light,
        ),
        ("theme-dark", tr("Тёмная", "Dark"), ThemePreference::Dark),
    ] {
        let selected = state.settings.theme == preference;
        let button = Button::new(id).label(name);
        let button = if selected {
            button.primary()
        } else {
            button.outline()
        };
        themes = themes.child(button.on_click(cx.listener(move |this, _, window, cx| {
            this.state.settings.theme = preference;
            let dark = match preference {
                ThemePreference::Dark => true,
                ThemePreference::Light => false,
                ThemePreference::System => crate::platform::system_dark(),
            };
            this.state.settings.dark_theme = dark;
            Theme::change(
                if dark {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                },
                Some(window),
                cx,
            );
            this.state.save_settings();
            cx.notify();
        })));
    }

    let mut languages = button_row();
    for (id, name, language) in [
        ("settings-ru", "Русский", Language::Ru),
        ("settings-en", "English", Language::En),
    ] {
        let selected = state.settings.language == language;
        let button = Button::new(id).label(name);
        let button = if selected {
            button.primary()
        } else {
            button.outline()
        };
        languages = languages.child(button.on_click(cx.listener(move |this, _, _, cx| {
            this.state.settings.language = language;
            i18n::set_language(language);
            this.state.save_settings();
            cx.notify();
        })));
    }

    section(tr("Внешний вид", "Appearance"))
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(Label::new(tr("Тема", "Theme")))
                .child(themes),
        )
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(Label::new(tr("Язык интерфейса", "Interface language")))
                .child(languages),
        )
        .into_any_element()
}
