use crate::root::view_data::ViewData;
use crate::{
    components::{page::page, stat_card::stat_card},
    i18n::tr,
    root::RootView,
    state::AppState,
};
use gpui_kit::{
    component::{label::Label, *},
    *,
};

fn duration(secs: u64) -> String {
    let (d, h, m) = (secs / 86400, secs / 3600 % 24, secs / 60 % 60);
    if d > 0 {
        crate::i18n::tr_args(
            "duration-days-hours",
            &[("days", d.into()), ("hours", h.into())],
        )
    } else if h > 0 {
        crate::i18n::tr_args(
            "duration-hours-minutes",
            &[("hours", h.into()), ("minutes", m.into())],
        )
    } else {
        crate::i18n::tr_args("duration-minutes", &[("minutes", m.into())])
    }
}

fn ago(secs: u64) -> String {
    if secs < 60 {
        tr("ui-just-now").into()
    } else {
        crate::i18n::tr_args("time-ago", &[("duration", duration(secs).into())])
    }
}

pub fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;
    let stats = &state.background.statistics;

    let last = stats
        .last_successful_test
        .map(|time| ago(data.now.saturating_sub(time)))
        .unwrap_or_else(|| tr("ui-not-yet").into());

    let row1 = div()
        .h_flex()
        .flex_wrap()
        .gap_3()
        .w_full()
        .child(stat_card(
            tr("ui-zui-uptime"),
            duration(stats.uptime_seconds),
            muted,
            border,
        ))
        .child(stat_card(
            tr("ui-last-successful-test"),
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
            tr("ui-strategy-changes"),
            stats.strategy_changes.to_string(),
            muted,
            border,
        ))
        .child(stat_card(
            tr("ui-winws-restarts"),
            stats.watchdog_restarts.to_string(),
            muted,
            border,
        ));

    let body = div()
        .v_flex()
        .gap_4()
        .w_full()
        .child(
            Label::new(tr("ui-statistics"))
                .text_lg()
                .font_weight(FontWeight::BOLD),
        )
        .child(row1)
        .child(row2);

    page(body)
}
