//! Создание окна, фоновые тики и завершение приложения.

use super::{OnboardingStep, RootView, Tab};
use crate::{i18n, state::AppState, tray::Tray};
use gpui_kit::component::*;
use gpui_kit::*;
use z_core::preferences::ThemePreference;

impl RootView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut state = AppState::new();
        i18n::set_language(&state.settings.language);
        let show_whats_new = state.settings.onboarded
            && state.settings.last_seen_version != env!("CARGO_PKG_VERSION");
        if state.settings.onboarded {
            state.start_selected_services();
        }
        let tray = match Tray::new(window) {
            Ok(tray) => Some(tray),
            Err(error) => {
                let message = crate::i18n::tr_args(
                    "tray-unavailable",
                    &[("error", (error).to_string().into())],
                );
                state.log.add(&message);
                state.settings_status = message.into();
                None
            }
        };
        let view = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            view.update(cx, |this, cx| this.on_close_requested(window, cx))
                .is_err()
        });
        Theme::change(
            if Self::dark_theme(state.settings.theme) {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            None,
            cx,
        );
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        this.state.background.hidden = this.tray.as_ref().is_some_and(Tray::hidden);
                        let dark = Self::dark_theme(this.state.settings.theme);
                        if dark != cx.theme().mode.is_dark() {
                            let mode = if dark {
                                ThemeMode::Dark
                            } else {
                                ThemeMode::Light
                            };
                            Theme::change(mode, None, cx);
                        }
                        if this.tray.as_ref().is_some_and(Tray::take_quit_request) {
                            cx.defer(|cx| cx.quit());
                            return;
                        }
                        this.state.poll();
                        this.refresh_and_notify(cx);
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        cx.on_app_quit(|this, _| {
            this.tray.take();
            this.state.shutdown();
            async {}
        })
        .detach();
        let strategy_filters = super::strategy_filters::StrategyFilters::new(window, cx);
        let mut view = Self {
            state,
            tab: Tab::Home,
            tray,
            onboarding_step: OnboardingStep::default(),
            show_whats_new,
            close_prompt_open: false,
            view_data: Default::default(),
            strategy_filters,
        };
        view.refresh_view_data();
        view
    }

    /// Observe external resources before rendering, preserving Home's reload rule.
    fn refresh_view_data(&mut self) {
        if self.tab == Tab::Home
            && self.state.settings.onboarded
            && !self.show_whats_new
            && self.state.zapret.installed()
            && self.state.zapret.strategies.is_empty()
        {
            self.state.zapret.reload_strategies();
        }
        self.state.reconcile_profile();
        self.view_data = super::view_data::ViewData::read(&mut self.state);
    }

    pub(super) fn refresh_and_notify(&mut self, cx: &mut Context<Self>) {
        self.refresh_view_data();
        cx.notify();
    }

    fn dark_theme(preference: ThemePreference) -> bool {
        match preference {
            ThemePreference::System => crate::platform::system_dark(),
            ThemePreference::Dark => true,
            ThemePreference::Light => false,
        }
    }
}
