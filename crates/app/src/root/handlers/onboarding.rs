//! Реакции GPUI на действия пользователя: onboarding.
use super::super::RootView;
use gpui_kit::*;
pub(crate) const STEPS: usize = 4;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum OnboardingStep {
    #[default]
    Language,
    Permissions,
    Services,
    Startup,
}

impl OnboardingStep {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Language => 0,
            Self::Permissions => 1,
            Self::Services => 2,
            Self::Startup => 3,
        }
    }

    pub(crate) fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Language,
            1 => Self::Permissions,
            2 => Self::Services,
            _ => Self::Startup,
        }
    }

    pub(crate) fn next(self) -> Self {
        Self::from_index((self.index() + 1).min(STEPS - 1))
    }
    pub(crate) fn previous(self) -> Self {
        Self::from_index(self.index().saturating_sub(1))
    }
}

impl RootView {
    pub(crate) fn on_onboarding_step(&mut self, index: usize, cx: &mut Context<Self>) {
        self.onboarding_step = OnboardingStep::from_index(index);
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_onboarding_back(&mut self, cx: &mut Context<Self>) {
        self.onboarding_step = self.onboarding_step.previous();
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_onboarding_next(&mut self, cx: &mut Context<Self>) {
        if self.onboarding_step == OnboardingStep::Startup {
            self.state.complete_onboarding();
        } else {
            self.state.save_settings();
            self.onboarding_step = self.onboarding_step.next();
        }
        self.refresh_and_notify(cx);
    }

    pub(crate) fn on_elevate(&mut self, cx: &mut Context<Self>) {
        if self.state.elevate() {
            cx.defer(|cx| cx.quit());
        }
        self.refresh_and_notify(cx);
    }
}
