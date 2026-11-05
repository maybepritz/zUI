//! Главная: работает ли обход, открываются ли сервисы и как менялась задержка за сутки.
use crate::components::{
    card::card,
    install_card::install_card,
    latency_chart::{Series, latency_chart},
    tone::{self, status_dot},
    update_banner::{app_update_banner, zapret_update_banner},
};
use crate::i18n::time::{ago, duration};
use crate::i18n::{tr, tr_args};
use crate::root::view_data::ViewData;
use crate::root::{RootView, Tab};
use crate::state::AppState;
use gpui_kit::component::{button::*, switch::Switch, *};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use z_core::health::BUCKET_SECONDS;
use z_core::services::{self, Check};

use crate::components::page::page;

/// Состояние одного сервиса на главной.
struct Row {
    id: String,
    /// Последняя проверка, если она уже была.
    last: Option<Check>,
    down: bool,
    /// Сколько секунд за сутки сервис не открывался.
    down_seconds: u64,
}

pub fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let running = data.zapret_running;
    let result = state.background.services.result.as_ref();
    let health = &state.background.health;
    let rows: Vec<Row> = state
        .settings
        .automation
        .services
        .iter()
        .map(|id| Row {
            id: id.clone(),
            last: result.and_then(|r| r.checks.iter().find(|c| &c.id == id).cloned()),
            down: running && health.down_since.contains_key(id),
            down_seconds: health.day(id).iter().filter(|b| b.down()).count() as u64
                * BUCKET_SECONDS,
        })
        .collect();
    let broken = rows.iter().find(|r| r.down);

    page(
        div()
            .v_flex()
            .gap_5()
            .when_some(app_update_banner(state, cx), |t, b| t.child(b))
            .when_some(zapret_update_banner(state, cx), |t, b| t.child(b))
            .when(!data.zapret_installed, |t| {
                t.child(install_card(state.zapret.installing(), cx))
            })
            .child(header(
                running,
                broken,
                &rows,
                result.map(|r| r.checked_at),
                data.now,
                cx,
            ))
            .child(chart(state, data, running, cx))
            .child(service_list(&rows, running, cx))
            .child(footer(state, running, broken.is_some(), cx)),
    )
}

fn header(
    running: bool,
    broken: Option<&Row>,
    rows: &[Row],
    checked_at: Option<u64>,
    now: u64,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let muted = cx.theme().muted_foreground;
    let (dot, title, hint) = match (running, broken) {
        (false, _) => (muted, tr("home-bypass-off"), tr("home-bypass-off-hint")),
        (true, Some(row)) => (
            tone::warning(cx),
            tr_args(
                "home-service-down",
                &[("service", services::name(&row.id).to_string().into())],
            ),
            tr("home-service-down-hint"),
        ),
        (true, None) => (
            tone::ok(cx),
            tr("home-bypass-on"),
            match checked_at {
                Some(at) if !rows.is_empty() => {
                    tr_args("home-checked", &[("ago", ago(at, now).into())])
                }
                _ => tr("home-checking"),
            },
        ),
    };
    div()
        .h_flex()
        .items_center()
        .gap_4()
        .child(
            div()
                .v_flex()
                .flex_1()
                .gap_1()
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_3()
                        .child(status_dot(dot))
                        .child(div().text_xl().font_weight(FontWeight::MEDIUM).child(title)),
                )
                .child(div().pl_5().text_sm().text_color(muted).child(hint)),
        )
        .child(
            Switch::new("bypass")
                .checked(running)
                .accessibility_label(tr("home-bypass-switch"))
                .on_click(cx.listener(|this, _, _, cx| this.on_toggle_zapret(cx))),
        )
}

fn chart(
    state: &AppState,
    data: &ViewData,
    running: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let theme = cx.theme();
    let muted = theme.muted_foreground;
    // Первая линия ведёт, остальные тише: цвет графика не должен спорить со статусом.
    let colors = [theme.foreground, muted, muted.opacity(0.6)];
    let ids = &state.settings.automation.services;
    let series: Vec<Series> = ids
        .iter()
        .zip(colors.iter().cycle())
        .map(|(id, color)| Series {
            color: *color,
            buckets: state.background.health.day(id).to_vec(),
        })
        .collect();
    let legend = ids.iter().zip(colors.iter().cycle()).fold(
        div().h_flex().gap_4(),
        |legend, (id, color)| {
            legend.child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .child(div().w_3().h(px(2.)).rounded_full().bg(*color))
                    .child(services::name(id).to_string()),
            )
        },
    );
    let axis = |text: String| div().text_xs().text_color(muted).child(text);
    card(cx)
        .gap_3()
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_4()
                .text_sm()
                .text_color(muted)
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground)
                        .child(tr("home-latency-day")),
                )
                .child(legend),
        )
        .child(latency_chart(
            series,
            data.now,
            running.then(|| tone::ok(cx)),
            tone::warning(cx),
            theme.border,
        ))
        .child(
            div()
                .h_flex()
                .justify_between()
                .child(axis(tr("home-axis-day-ago")))
                .child(axis(tr("home-axis-12h")))
                .child(axis(tr("home-axis-now"))),
        )
}

fn service_list(rows: &[Row], running: bool, cx: &mut Context<RootView>) -> impl IntoElement {
    let theme = cx.theme();
    let muted = theme.muted_foreground;
    let mut list = card(cx).p_0();
    for (index, row) in rows.iter().enumerate() {
        let connected = row.last.as_ref().map(|c| c.availability.connected());
        let (dot, detail) = match (running, connected) {
            (false, _) => (muted, tr("home-service-not-checked")),
            (true, None) => (muted, tr("home-checking")),
            (true, Some(_)) if row.down => (tone::warning(cx), tr("home-service-not-responding")),
            (true, Some(_)) if row.down_seconds > 0 => (
                tone::ok(cx),
                tr_args(
                    "home-service-outages",
                    &[("duration", duration(row.down_seconds).into())],
                ),
            ),
            (true, Some(_)) => (tone::ok(cx), tr("home-service-no-outages")),
        };
        let latency = match (&row.last, running && !row.down) {
            (Some(check), true) if check.availability.connected() => {
                tr_args("home-latency-ms", &[("ms", check.latency_ms.into())])
            }
            _ => "—".into(),
        };
        list = list.child(
            div()
                .h_flex()
                .h_11()
                .px_4()
                .gap_3()
                .items_center()
                .when(index > 0, |d| d.border_t_1().border_color(theme.border))
                .child(status_dot(dot))
                .child(div().flex_1().child(services::name(&row.id).to_string()))
                .child(div().text_sm().text_color(muted).child(detail))
                .child(
                    div()
                        .w_20()
                        .text_right()
                        .text_sm()
                        .font_family("Consolas")
                        .child(latency),
                ),
        );
    }
    list
}

fn footer(
    state: &AppState,
    running: bool,
    broken: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let muted = cx.theme().muted_foreground;
    let strategy = state
        .zapret
        .selected()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| tr("home-strategy-none"));
    let action = if broken {
        Button::new("home-retest")
            .primary()
            .label(tr("home-retest"))
            .on_click(cx.listener(|this, _, _, cx| this.on_start_test(cx)))
    } else {
        Button::new("home-check")
            .outline()
            .label(tr("home-check-now"))
            .disabled(!running || state.background.services.busy())
            .on_click(cx.listener(|this, _, _, cx| this.on_check_services(cx)))
    };
    div()
        .h_flex()
        .items_center()
        .gap_3()
        .text_sm()
        .text_color(muted)
        .child(
            div()
                .flex_1()
                .h_flex()
                .gap_1()
                .child(tr_args("home-strategy", &[("strategy", strategy.into())]))
                .child(
                    Button::new("home-change-strategy")
                        .link()
                        .small()
                        .label(tr("home-change"))
                        .on_click(
                            cx.listener(|this, _, _, cx| this.on_navigate(Tab::Strategies, cx)),
                        ),
                ),
        )
        .child(action)
}
