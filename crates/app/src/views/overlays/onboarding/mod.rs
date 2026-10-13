//! Разметка мастера первого запуска.
use crate::{components::section::button_row, i18n::tr, root::RootView};
use crate::{root::view_data::ViewData, state::AppState};
use gpui_kit::{
    component::{
        button::*,
        label::Label,
        progress::Progress,
        scroll::ScrollableElement as _,
        stepper::{Stepper, StepperItem},
        *,
    },
    *,
};

use crate::root::{OnboardingStep as Step, STEPS};

mod language;
mod permissions;
mod services;
mod startup;
use language::language;
use permissions::permissions;
use services::services;
use startup::startup;

pub fn render(
    state: &AppState,
    step: Step,
    data: &ViewData,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let idx = step.index();
    let busy = state.zapret.installing() || state.telegram.installing();

    let content = match step {
        Step::Language => language(&state.settings.language, cx),
        Step::Permissions => permissions(data.administrator, cx),
        Step::Services => services(state, data, cx),
        Step::Startup => startup(state, cx),
    };

    // Степпер: нельзя перепрыгнуть вперёд, назад — можно
    let titles = [
        (IconName::Globe, tr("ui-language")),
        (IconName::Info, tr("ui-permissions")),
        (IconName::Settings, tr("ui-services")),
        (IconName::CircleCheck, tr("ui-startup")),
    ];
    let items = titles.into_iter().enumerate().map(|(i, (icon, title))| {
        StepperItem::new()
            .icon(icon)
            .disabled(i > idx || busy)
            .child(title)
    });

    let stepper = Stepper::new("wizard-stepper")
        .w_full()
        .selected_index(idx)
        .items(items)
        .on_click(cx.listener(|this, i: &usize, _, cx| {
            this.on_onboarding_step(*i, cx);
        }));

    let progress = Progress::new("wizard-progress")
        .small()
        .value((idx + 1) as f32 / STEPS as f32 * 100.0);

    let card = div()
        .v_flex()
        .gap_4()
        .p_4()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .child(content);

    let footer = div()
        .h_flex()
        .justify_between()
        .items_center()
        .child(Label::new(state.settings_status.clone()).text_sm())
        .child(
            button_row()
                .child(
                    Button::new("wizard-back")
                        .outline()
                        .label(tr("ui-back"))
                        .disabled(step == Step::Language || busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_onboarding_back(cx);
                        })),
                )
                .child(
                    Button::new("wizard-next")
                        .primary()
                        .label(if step == Step::Startup {
                            tr("ui-finish")
                        } else {
                            tr("ui-next")
                        })
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_onboarding_next(cx);
                        })),
                ),
        );

    div()
        .v_flex()
        .size_full()
        .gap_5()
        .p_6()
        .overflow_y_scrollbar()
        .child(
            div()
                .h_flex()
                .justify_between()
                .items_center()
                .child(
                    Label::new(tr("ui-first-time-setup"))
                        .text_lg()
                        .font_weight(FontWeight::BOLD),
                )
                .child(Label::new(format!("{} / {}", idx + 1, STEPS)).text_sm()),
        )
        .child(stepper)
        .child(progress)
        .child(card)
        .child(footer)
        .into_any_element()
}
