use super::formatting::ago;
use crate::{
    components::{empty_state::empty_state, tone},
    i18n::{tr, tr_args},
    root::{RootView, Tab, view_data::ViewData},
    state::AppState,
};
use gpui_kit::{
    assets::IconName,
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> Div {
    let stats = &state.background.statistics;
    let muted = cx.theme().muted_foreground;
    let busy = state.diagnostics.busy();
    let mut card = crate::components::cards::card::card(cx)
        .gap_3()
        .bg(cx.theme().muted.opacity(0.25))
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(IconName::Activity).size_4().text_color(muted))
                        .child(
                            Label::new(tr("statistics-last-check"))
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM),
                        ),
                )
                .child(
                    Button::new("statistics-open-testing")
                        .outline()
                        .label(tr("statistics-open-testing"))
                        .icon(IconName::ArrowRight)
                        .on_click(
                            cx.listener(|this, _, _, cx| this.on_navigate(Tab::Strategies, cx)),
                        ),
                ),
        );
    if busy {
        let (done, total) = state.diagnostics.progress();
        card = card.child(
            Label::new(tr_args(
                "test-progress",
                &[("done", done.into()), ("total", total.into())],
            ))
            .text_xs()
            .text_color(muted),
        );
    }
    if let Some(test) = &stats.last_test {
        let status = if !test.completed {
            "statistics-check-cancelled"
        } else if test
            .best
            .as_ref()
            .is_some_and(|best| best.fully_available())
        {
            "statistics-check-success"
        } else if test.best.is_some() {
            "statistics-check-partial"
        } else {
            "statistics-check-failed"
        };
        // Цвет статуса есть только у плашки результата; остальной интерфейс нейтральный.
        let color = if test.completed
            && test
                .best
                .as_ref()
                .is_some_and(|best| best.fully_available())
        {
            tone::ok(cx)
        } else {
            cx.theme().muted_foreground
        };
        card = card
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(color)
                    .child(tr(status)),
            )
            .child(
                Label::new(tr_args(
                    "statistics-check-meta",
                    &[
                        ("time", ago(test.finished_at, data.now).into()),
                        ("count", test.strategies_tested.into()),
                    ],
                ))
                .text_xs()
                .text_color(muted),
            );
        if let Some(best) = &test.best {
            card = card
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            Label::new(tr(if test.completed {
                                "statistics-best-strategy"
                            } else {
                                "statistics-best-so-far"
                            }))
                            .text_xs()
                            .text_color(muted),
                        )
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::MEDIUM)
                                .child(best.name.clone()),
                        ),
                )
                .child(
                    div()
                        .h_flex()
                        .flex_wrap()
                        .gap_4()
                        .child(
                            Label::new(tr_args(
                                "statistics-targets",
                                &[("passed", best.passed.into()), ("total", best.total.into())],
                            ))
                            .text_sm(),
                        )
                        .child(
                            Label::new(tr_args(
                                "statistics-latency",
                                &[("ms", best.avg_ms.into())],
                            ))
                            .text_sm(),
                        ),
                );
        }
        card = card.child(
            Label::new(tr("statistics-snapshot-hint"))
                .text_xs()
                .text_color(muted),
        );
    } else {
        card = card.child(empty_state(
            IconName::Activity,
            tr("statistics-check-empty"),
            tr("statistics-check-empty-hint"),
        ));
        // В старой статистике есть только время, поэтому подробный результат не придумываем.
        if let Some(timestamp) = stats.last_successful_test {
            card = card.child(
                Label::new(tr_args(
                    "statistics-legacy-check",
                    &[("time", ago(timestamp, data.now).into())],
                ))
                .text_xs()
                .text_color(muted),
            );
        }
    }
    card
}
