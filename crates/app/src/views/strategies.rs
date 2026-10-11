use crate::i18n::tr;
use crate::components::section::{button_row, section};
use crate::{root::RootView, state::AppState};
use gpui_kit::component::button::*;
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
use gpui_kit::component::label::Label;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use z_core::zapret::diagnostics::Score;

fn intro() -> &'static str {
    tr("Выберите наиболее подходящую стратегию под вашего провайдера. При смене активной стратегии работающий Zapret перезапустится автоматически.", "Choose a strategy for your ISP. Changing the active strategy automatically restarts Zapret.")
}
fn test_hint() -> &'static str {
    tr("Перебор стратегий с проверкой Discord и YouTube. Интернет на время теста нестабилен, Zapret будет остановлен.", "Tests strategies against Discord and YouTube. Zapret temporarily stops and connectivity may be unstable.")
}

fn green() -> Hsla {
    hsla(142.0 / 360.0, 0.76, 0.45, 1.0)
}

fn red() -> Hsla {
    hsla(0.0, 0.70, 0.50, 1.0)
}

pub fn render(state: &mut AppState, cx: &mut Context<RootView>) -> AnyElement {
    let is_running = state.zapret.is_running();
    let current = state.zapret.selected_index;
    let muted = cx.theme().muted_foreground;
    let scores = state.diagnostics.rows();

    let cards: Vec<AnyElement> = state
        .zapret
        .strategies
        .iter()
        .enumerate()
        .map(|(ix, s)| {
            let score = scores.iter().find(|r| r.file_name == s.file_name);
            strategy_card(
                ix,
                &s.name,
                &s.file_name,
                ix == current,
                is_running,
                state.zapret.locked || state.zapret.installing(),
                score,
                muted,
                cx,
            )
        })
        .collect();

    div()
        .v_flex()
        .size_full()
        .gap_4()
        .overflow_y_scrollbar()
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    Label::new(tr("Стратегии обхода блокировок DPI", "DPI bypass strategies"))
                        .text_lg()
                        .font_weight(FontWeight::BOLD),
                )
                .child(Label::new(intro()).text_xs().text_color(muted)),
        )
        .child(test_section(state, muted, cx))
        .children(cards)
        .into_any_element()
}

fn test_section(state: &AppState, muted: Hsla, cx: &mut Context<RootView>) -> impl IntoElement {
    let diag = &state.diagnostics;
    let busy = diag.busy();
    let (done, total) = diag.progress();
    let has_best = diag.best().is_some();
    let is_admin = z_core::dns::is_elevated();

    let hint = if busy {
        format!("Проверено {done} из {total}…")
    } else if !is_admin {
        tr("Для теста нужны права администратора.", "Testing requires administrator privileges.").to_string()
    } else {
        test_hint().to_string()
    };

    section(tr("Подбор стратегии", "Strategy testing"))
        .child(Label::new(hint).text_xs().text_color(muted))
        .child(
            button_row()
                .child(
                    Button::new("diag-start")
                        .primary()
                        .label(tr("Проверить стратегии", "Test strategies"))
                        .disabled(busy || !is_admin || state.zapret.installing())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.start_test(false);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("diag-cancel")
                        .outline()
                        .label(tr("Отмена", "Cancel"))
                        .disabled(!busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.diagnostics.cancel();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("diag-best")
                        .outline()
                        .label(tr("Выбрать лучшую", "Select best"))
                        .disabled(busy || !has_best || state.zapret.installing())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.apply_best_strategy();
                            cx.notify();
                        })),
                ),
        )
}

fn score_label(score: &Score) -> (String, Hsla) {
    match &score.error {
        Some(_) => (tr("не запустилась", "failed to start").to_string(), red()),
        None if score.is_ok() => (
            format!("{}/{}, {} мс", score.passed, score.total, score.avg_ms),
            green(),
        ),
        None if score.passed == 0 => (format!("0/{}", score.total), red()),
        None => (
            format!("{}/{}, {} мс", score.passed, score.total, score.avg_ms),
            hsla(38.0 / 360.0, 0.92, 0.50, 1.0),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn strategy_card(
    ix: usize,
    name: &str,
    file_name: &str,
    selected: bool,
    running: bool,
    locked: bool,
    score: Option<&Score>,
    muted: Hsla,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let right = if selected {
        div().into_any_element()
    } else {
        Button::new(format!("select-strat-{ix}"))
            .outline()
            .small()
            .label(tr("Выбрать", "Select"))
            .disabled(locked)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.state.select_strategy(ix);
                cx.notify();
            }))
            .into_any_element()
    };

    GroupBox::new()
        .outline()
        .gap_2()
        .child(
            div()
                .h_flex()
                .w_full()
                .gap_3()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .h_flex()
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
                                        tr("✓ Работает", "Running")
                                    } else {
                                        tr("✓ Выбрана", "Selected")
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
                        }),
                )
                .child(right),
        )
        .into_any_element()
}
