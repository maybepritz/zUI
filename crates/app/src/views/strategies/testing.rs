//! Разметка стратегий и результатов тестирования — testing.
use super::test_hint;
use crate::components::section::{button_row, section};
use crate::{i18n::tr, root::RootView, root::view_data::ViewData, state::AppState};
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn test_section(
    state: &AppState,
    data: &ViewData,
    muted: Hsla,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let diag = &state.diagnostics;
    let busy = diag.busy() || state.profile_busy();
    let (done, total) = diag.progress();
    let has_best = diag.best().is_some();
    let is_admin = data.administrator;

    let hint = if diag.busy() {
        crate::i18n::tr_args(
            "test-progress",
            &[("done", (done).into()), ("total", (total).into())],
        )
    } else if !is_admin {
        tr("ui-testing-requires-administrator-privileges").to_string()
    } else {
        test_hint().to_string()
    };

    section(tr("ui-strategy-testing"))
        .child(Label::new(hint).text_xs().text_color(muted))
        .child(
            button_row()
                .child(
                    Button::new("diag-start")
                        .primary()
                        .label(tr("ui-test-strategies"))
                        .disabled(busy || !is_admin || state.zapret.installing())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_start_test(cx);
                        })),
                )
                .child(
                    Button::new("diag-cancel")
                        .outline()
                        .label(tr("ui-cancel"))
                        .disabled(!busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_cancel_test(cx);
                        })),
                )
                .child(
                    Button::new("diag-best")
                        .outline()
                        .label(tr("ui-select-best"))
                        .disabled(busy || !has_best || state.zapret.installing())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_apply_best_strategy(cx);
                        })),
                ),
        )
}
