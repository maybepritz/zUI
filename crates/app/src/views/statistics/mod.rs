//! Разметка статистики. Отрисовка читает сохранённые снимки и не меняет состояние.

mod history;
mod summary;

use crate::{
    components::{page::page, stat_card::stat_card},
    i18n::tr,
    root::{RootView, view_data::ViewData},
    state::AppState,
};
use gpui_kit::{
    assets::IconName,
    component::{label::Label, *},
    *,
};

pub fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let stats = &state.background.statistics;
    let muted = cx.theme().muted_foreground;
    let metrics = div()
        .h_flex()
        .flex_wrap()
        .gap_3()
        .w_full()
        .child(stat_card(
            tr("ui-zui-uptime"),
            crate::i18n::time::duration(stats.uptime_seconds),
            tr("statistics-uptime-hint"),
            IconName::Clock,
            cx,
        ))
        .child(stat_card(
            tr("ui-strategy-changes"),
            stats.strategy_changes.to_string(),
            tr("statistics-changes-hint"),
            IconName::ArrowRightLeft,
            cx,
        ))
        .child(stat_card(
            tr("ui-winws-restarts"),
            stats.watchdog_restarts.to_string(),
            tr("statistics-restarts-hint"),
            IconName::RotateCcw,
            cx,
        ));

    page(
        div()
            .v_flex()
            .gap_4()
            .w_full()
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(
                        Label::new(tr("ui-statistics"))
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM),
                    )
                    .child(
                        Label::new(tr("statistics-intro"))
                            .text_xs()
                            .text_color(muted),
                    ),
            )
            .child(summary::render(state, data, cx))
            .child(metrics)
            .child(history::render(stats, data.now, cx)),
    )
}
