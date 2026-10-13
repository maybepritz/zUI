//! Реакции GPUI на действия пользователя: settings.
use super::super::RootView;
use crate::i18n;
use crate::state::Preference;
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;
use z_core::preferences::{Language, ThemePreference};

impl RootView {
    pub(crate) fn on_preference(
        &mut self,
        preference: Preference,
        value: bool,
        cx: &mut Context<Self>,
    ) {
        self.state.set_preference(preference, value);
        self.refresh_and_notify(cx);
    }
    pub(crate) fn on_autostart(&mut self, checked: bool, cx: &mut Context<Self>) {
        self.state.set_autostart(checked);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_reopen_onboarding(&mut self, cx: &mut Context<Self>) {
        self.state.reopen_onboarding();
        self.onboarding_step = Default::default();
        self.state.save_settings();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_theme(
        &mut self,
        preference: ThemePreference,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dark = match preference {
            ThemePreference::Dark => true,
            ThemePreference::Light => false,
            ThemePreference::System => crate::platform::system_dark(),
        };
        self.state.set_theme(preference, dark);
        Theme::change(
            if dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            Some(window),
            cx,
        );
        self.state.save_settings();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_language(
        &mut self,
        language: Language,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.state.set_language(language.clone());
        i18n::set_language(&language);
        self.strategy_filters.input.update(cx, |input, cx| {
            input.set_placeholder(i18n::tr("strategy-search-placeholder"), window, cx);
        });
        if let Some(tray) = &self.tray {
            tray.refresh_language();
        }
        self.state.save_settings();
        self.refresh_and_notify(cx);
    }
}
