//! Разметка мастера первого запуска — startup.
use crate::{components::section::button_row, i18n::tr, root::RootView, state::AppState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn startup(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let on = state.settings.autostart;
    div()
        .v_flex()
        .gap_3()
        .child(Label::new(tr("ui-start-zui-when-you-sign-in-to")))
        .child(
            button_row()
                .child(
                    Button::new("wizard-autostart-on")
                        .label(tr("ui-enable"))
                        .when(on, |b| b.primary())
                        .when(!on, |b| b.outline())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_autostart(true, cx);
                        })),
                )
                .child(
                    Button::new("wizard-autostart-off")
                        .label(tr("ui-disable"))
                        .when(!on, |b| b.primary())
                        .when(on, |b| b.outline())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_autostart(false, cx);
                        })),
                ),
        )
        .child(
            Label::new(tr(
                "ui-autostart-does-not-elevate-privileges-zapret-requires",
            ))
            .text_sm(),
        )
        .into_any_element()
}
