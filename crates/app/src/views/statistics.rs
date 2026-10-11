use crate::{components::hint::hint, i18n::tr, root::RootView, state::AppState};
use gpui_kit::{
    component::{label::Label, scroll::ScrollableElement as _, *},
    *,
};

fn duration(secs: u64) -> String {
    let (d, h, m) = (secs / 86400, secs / 3600 % 24, secs / 60 % 60);
    let (dl, hl, ml) = (tr("д", "d"), tr("ч", "h"), tr("мин", "min"));
    if d > 0 {
        format!("{d} {dl} {h} {hl}")
    } else if h > 0 {
        format!("{h} {hl} {m} {ml}")
    } else {
        format!("{m} {ml}")
    }
}

fn ago(secs: u64) -> String {
    if secs < 60 {
        tr("только что", "just now").into()
    } else {
        format!("{} {}", duration(secs), tr("назад", "ago"))
    }
}

fn stat_card(title: &str, value: String, muted: Hsla, border: Hsla) -> Div {
    div()
        .v_flex()
        .gap_1()
        .p_4()
        .flex_1()
        .min_w(px(200.))
        .rounded_lg()
        .border_1()
        .border_color(border)
        .child(hint(Label::new(title.to_string()), muted))
        .child(div().text_2xl().font_weight(FontWeight::BOLD).child(value))
}

pub fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;
    let stats = &state.background.statistics;

    let last = stats
        .last_successful_test
        .map(|time| ago(z_core::statistics::now().saturating_sub(time)))
        .unwrap_or_else(|| tr("Ещё не было", "Not yet").into());

    let row1 = div()
        .h_flex()
        .flex_wrap()
        .gap_3()
        .w_full()
        .child(stat_card(
            tr("Время работы zUI", "zUI uptime"),
            duration(stats.uptime_seconds),
            muted,
            border,
        ))
        .child(stat_card(
            tr("Последний успешный тест", "Last successful test"),
            last,
            muted,
            border,
        ));

    let row2 = div()
        .h_flex()
        .flex_wrap()
        .gap_3()
        .w_full()
        .child(stat_card(
            tr("Смен стратегии", "Strategy changes"),
            stats.strategy_changes.to_string(),
            muted,
            border,
        ))
        .child(stat_card(
            tr("Перезапусков winws", "winws restarts"),
            stats.watchdog_restarts.to_string(),
            muted,
            border,
        ));

    let body = div()
        .v_flex()
        .gap_4()
        .w_full()
        .max_w(px(720.))
        .flex_shrink_0()
        .child(
            Label::new(tr("Статистика", "Statistics"))
                .text_lg()
                .font_weight(FontWeight::BOLD),
        )
        .child(row1)
        .child(row2);

    div()
        .size_full()
        .p_6()
        .h_flex()
        .justify_center()
        .overflow_y_scrollbar()
        .child(body)
        .into_any_element()
}
