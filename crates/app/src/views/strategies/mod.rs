//! Разметка стратегий и результатов тестирования.
use crate::components::page::page;
use crate::i18n::tr;
use crate::root::strategy_filters::StrategyFilters;
use crate::root::view_data::ViewData;
use crate::{root::RootView, state::AppState};
use gpui_kit::component::label::Label;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

mod card;
mod filters;
mod testing;
use card::strategy_card;
use testing::test_section;

fn intro() -> String {
    tr("ui-choose-a-strategy-for-your-isp-changing")
}

fn test_hint() -> String {
    tr("ui-tests-strategies-against-discord-and-youtube-zapret")
}

pub fn render(
    state: &AppState,
    data: &ViewData,
    filters: &StrategyFilters,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let is_running = data.zapret_running;
    let current = state.zapret.selected_index;
    let muted = cx.theme().muted_foreground;
    let scores = &data.scores;
    let locked = state.zapret.locked || state.zapret.installing() || state.profile_busy();
    let total = state.zapret.strategies.len();
    let query = filters.query.to_lowercase();
    let terms: Vec<_> = query.split_whitespace().collect();

    // Keep controller indices attached to each row before changing display order.
    let mut strategies: Vec<_> = state
        .zapret
        .strategies
        .iter()
        .enumerate()
        .map(|(ix, s)| {
            let favorite = state
                .settings
                .favorite_strategies
                .iter()
                .any(|file| file == &s.file_name);
            (ix, s, favorite)
        })
        .filter(|(_, strategy, favorite)| {
            if filters.favorites_only && !*favorite {
                return false;
            }
            let name = strategy.name.to_lowercase();
            let file = strategy.file_name.to_lowercase();
            terms
                .iter()
                .all(|term| name.contains(*term) || file.contains(*term))
        })
        .collect();

    // Stable sort: favorites first, original strategy order within each group.
    strategies.sort_by_key(|entry| !entry.2);
    let shown = strategies.len();

    let cards: Vec<AnyElement> = strategies
        .into_iter()
        .map(|(ix, s, favorite)| {
            let score = scores.iter().find(|r| r.file_name == s.file_name);
            strategy_card(
                ix,
                &s.name,
                &s.file_name,
                favorite,
                ix == current,
                is_running,
                locked,
                score,
                muted,
                cx,
            )
        })
        .collect();

    page(
        div()
            .v_flex()
            .gap_4()
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(
                        Label::new(tr("ui-dpi-bypass-strategies"))
                            .text_lg()
                            .font_weight(FontWeight::BOLD),
                    )
                    .child(Label::new(intro()).text_xs().text_color(muted)),
            )
            .child(test_section(state, data, muted, cx))
            .child(filters::render(filters, shown, total, cx))
            .when(shown == 0, |content| {
                content.child(filters::empty_state(filters, total, cx))
            })
            .children(cards),
    )
}
