//! Разметка статистики. Отрисовка читает сохранённые снимки и не меняет состояние.

mod history;
mod summary;

use crate::{
    components::{page::page, page_title::page_title, stat_card::stat_card},
    i18n::tr,
    root::{RootView, view_data::ViewData},
    state::AppState,
};
use gpui_kit::{component::*, *};

pub fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let stats = &state.background.statistics;
    let metrics = div()
        .h_flex()
        // Плитки одной высоты, даже если подпись у одной из них в две строки.
        .items_stretch()
        .flex_wrap()
        .gap_3()
        .w_full()
        .child(stat_card(
            tr("ui-zui-uptime"),
            crate::i18n::time::duration(stats.uptime_seconds),
            tr("statistics-uptime-hint"),
            cx,
        ))
        .child(stat_card(
            tr("ui-strategy-changes"),
            stats.strategy_changes.to_string(),
            tr("statistics-changes-hint"),
            cx,
        ))
        .child(stat_card(
            tr("ui-winws-restarts"),
            stats.watchdog_restarts.to_string(),
            tr("statistics-restarts-hint"),
            cx,
        ));

    page(
        div()
            .v_flex()
            .gap_6()
            .w_full()
            .child(page_title(
                tr("ui-statistics"),
                None,
                Some(tr("statistics-intro").into()),
                cx,
            ))
            .child(summary::render(state, data, cx))
            .child(metrics)
            .child(history::render(stats, data.now, cx)),
    )
}
