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

    /// «Вместе с Windows» для человека значит «обход включится сам», поэтому вместе
    /// с автозапуском включается и запуск Zapret. Выключение его не трогает: zUI,
    /// открытый вручную, ведёт себя как раньше.
    pub(crate) fn on_start_with_windows(&mut self, checked: bool, cx: &mut Context<Self>) {
        self.state.set_autostart(checked);
        if checked && self.state.settings.autostart {
            self.state.set_preference(Preference::StartupZapret, true);
        }
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_close_behavior(
        &mut self,
        behavior: crate::state::CloseBehavior,
        cx: &mut Context<Self>,
    ) {
        self.state.set_close_behavior(behavior);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_notifications(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.state.set_notifications(enabled);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_chart_range(&mut self, hour: bool, cx: &mut Context<Self>) {
        self.chart_hour = hour;
        cx.notify();
    }

    /// «Понятно» на плашке о VPN: прячет её, пока текст тот же.
    pub(crate) fn on_dismiss_vpn(&mut self, text: String, cx: &mut Context<Self>) {
        self.vpn_dismissed = Some(text);
        cx.notify();
    }

    pub(crate) fn on_toggle_advanced_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_advanced = !self.settings_advanced;
        cx.notify();
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
        crate::platform::set_title_bar_dark(self.hwnd, dark);
        Theme::change(
            if dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            Some(window),
            cx,
        );
        crate::components::palette::apply(cx);
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
