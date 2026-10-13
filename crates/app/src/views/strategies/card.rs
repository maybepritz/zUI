//! Разметка стратегий и результатов тестирования — card.
use crate::components::list_card::list_card;
use crate::components::rows::favorite_button::{favorite_button, favorite_color};
use crate::{i18n::tr, root::RootView};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};
use z_core::zapret::diagnostics::Score;

fn score_label(score: &Score) -> (String, Hsla) {
    match &score.error {
        Some(_) => (tr("ui-failed-to-start").to_string(), red()),
        None if score.is_ok() => (
            crate::i18n::tr_args(
                "test-score",
                &[
                    ("passed", (score.passed).into()),
                    ("total", (score.total).into()),
                    ("latency", (score.avg_ms).into()),
                ],
            ),
            green(),
        ),
        None if score.passed == 0 => (format!("0/{}", score.total), red()),
        None => (
            crate::i18n::tr_args(
                "test-score",
                &[
                    ("passed", (score.passed).into()),
                    ("total", (score.total).into()),
                    ("latency", (score.avg_ms).into()),
                ],
            ),
            hsla(38.0 / 360.0, 0.92, 0.50, 1.0),
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
    let favorite_ink = favorite_color(cx);

    list_card(
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
                    .font_weight(FontWeight::BOLD),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(format!("({file_name})")),
            )
            .when(selected, |this| {
                this.child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(green())
                        .child(if running {
                            tr("ui-running-2")
                        } else {
                            tr("ui-selected")
                        }),
                )
            })
            .when_some(score.map(score_label), |this, (text, color)| {
                this.child(
                    div()
                        .text_xs()
                        .font_family("Consolas")
                        .text_color(color)
                        .child(text),
                )
            })
            .when(favorite, |this| {
                this.child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(favorite_ink)
                        .bg(hsla(40.0 / 360.0, 0.85, 0.55, 0.10))
                        .rounded_md()
                        .px_2()
                        .py_0p5()
                        .child(tr("favorite-label")),
                )
            }),
        right,
    )
}

fn green() -> Hsla {
    hsla(142.0 / 360.0, 0.76, 0.45, 1.0)
}

fn red() -> Hsla {
    hsla(0.0, 0.70, 0.50, 1.0)
}
