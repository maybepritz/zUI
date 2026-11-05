use crate::i18n::time::ago;
use crate::{
    components::empty_state::empty_state,
    i18n::{tr, tr_args},
};
use gpui_kit::{
    assets::IconName,
    component::{label::Label, *},
    *,
};
use z_core::statistics::{Statistics, StatisticsEventKind};

/// Сначала новые записи; размер сохранённой истории ограничен отдельно в z-core.
pub(super) fn render(stats: &Statistics, now: u64, cx: &App) -> Div {
    let muted = cx.theme().muted_foreground;
    let mut rows = div().v_flex().w_full();
    for (index, event) in stats.history.iter().rev().take(20).enumerate() {
        let (icon, message) = match &event.kind {
            StatisticsEventKind::StrategyChanged { strategy } => (
                IconName::ArrowRightLeft,
                tr_args(
                    "statistics-event-strategy",
                    &[("strategy", strategy.clone().into())],
                ),
            ),
            StatisticsEventKind::TestFinished { completed } => (
                IconName::Activity,
                tr(if *completed {
                    "statistics-event-test"
                } else {
                    "statistics-event-cancelled"
                }),
            ),
            StatisticsEventKind::ServiceDown { service, from, to } => (
                IconName::TriangleAlert,
                tr_args(
                    "statistics-event-service-down",
                    &[
                        (
                            "service",
                            z_core::services::name(service).to_string().into(),
                        ),
                        (
                            "duration",
                            crate::i18n::time::duration(to.saturating_sub(*from)).into(),
                        ),
                    ],
                ),
            ),
            StatisticsEventKind::Recovery { restarted } => (
                IconName::RotateCcw,
                tr(if *restarted {
                    "statistics-event-recovered"
                } else {
                    "statistics-event-recovery-failed"
                }),
            ),
        };
        let mut row = div().h_flex().items_start().gap_3().py_3();
        if index > 0 {
            row = row.border_t_1().border_color(cx.theme().border);
        }
        rows = rows.child(
            row.child(Icon::new(icon).size_4().text_color(muted).flex_shrink_0())
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .flex_1()
                        .min_w_0()
                        .child(div().text_sm().child(message))
                        .child(
                            Label::new(ago(event.timestamp, now))
                                .text_xs()
                                .text_color(muted),
                        ),
                ),
        );
    }
    if stats.history.is_empty() {
        rows = rows.child(empty_state(
            IconName::ScrollText,
            tr("statistics-history-empty"),
            tr("statistics-history-empty-hint"),
        ));
    }
    crate::components::cards::card::card(cx)
        .gap_1()
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(Icon::new(IconName::ScrollText).size_4().text_color(muted))
                .child(
                    Label::new(tr("statistics-history"))
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM),
                ),
        )
        .child(
            Label::new(tr("statistics-history-hint"))
                .text_xs()
                .text_color(muted),
        )
        .child(rows)
}
