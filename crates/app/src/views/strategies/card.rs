//! Разметка стратегий и результатов тестирования — card.
use crate::components::list_card::{list_card, selected_list_card};
use crate::components::rows::favorite_button::favorite_button;
use crate::components::tone::{self, status_dot};
use crate::{i18n::tr, root::RootView};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};
use z_core::zapret::diagnostics::Score;

fn score_label(score: &Score, cx: &App) -> (String, Hsla) {
    match &score.error {
        Some(_) => (tr("ui-failed-to-start").to_string(), tone::error(cx)),
        None if score.is_ok() => (
            crate::i18n::tr_args(
                "test-score",
                &[
                    ("passed", score.passed.into()),
                    ("total", score.total.into()),
                    ("latency", score.avg_ms.into()),
                ],
            ),
            tone::ok(cx),
        ),
        None if score.passed == 0 => (format!("0/{}", score.total), tone::error(cx)),
        None => (
            crate::i18n::tr_args(
                "test-score",
                &[
                    ("passed", score.passed.into()),
                    ("total", score.total.into()),
                    ("latency", score.avg_ms.into()),
                ],
            ),
            tone::warning(cx),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn strategy_card(
    ix: usize,
    name: &str,
    file_name: &str,
    favorite: bool,
    selected: bool,
    running: bool,
    locked: bool,
    score: Option<&Score>,
    muted: Hsla,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let select = if selected {
        div().into_any_element()
    } else {
        Button::new(format!("select-strat-{ix}"))
            .outline()
            .small()
            .label(tr("ui-select"))
            .disabled(locked)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.on_select_strategy(ix, cx);
            }))
            .into_any_element()
    };

    let favorite_file = file_name.to_owned();
    let star = favorite_button(
        format!("favorite-strat-{file_name}"),
        favorite,
        tr(if favorite {
            "favorite-remove"
        } else {
            "favorite-add"
        }),
        cx,
    )
    .on_click(cx.listener(move |this, _, _, cx| {
        this.on_toggle_favorite(&favorite_file, cx);
    }));
    let right = div()
        .h_flex()
        .items_center()
        .flex_shrink_0()
        .gap_2()
        .child(star)
        .child(select);
    let score = score.map(|score| score_label(score, cx));
    let state_dot = if running { tone::ok(cx) } else { muted };

    let frame = if selected {
        selected_list_card
    } else {
        list_card
    };
    frame(
        div()
            .h_flex()
            .flex_1()
            .min_w_0()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(
                Label::new(name.to_string())
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .font_family(tone::MONO)
                    .child(file_name.to_string()),
            )
            .when(selected, |this| {
                this.child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_1()
                        .text_xs()
                        .child(status_dot(state_dot))
                        .child(if running {
                            tr("strategy-running")
                        } else {
                            tr("ui-selected")
                        }),
                )
            })
            .when_some(score, |this, (text, color)| {
                this.child(
                    div()
                        .text_xs()
                        .font_family(crate::components::tone::MONO)
                        .text_color(color)
                        .child(text),
                )
            }),
        right,
    )
}
