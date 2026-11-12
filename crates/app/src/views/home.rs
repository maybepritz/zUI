//! Главная: работает ли обход, открываются ли сервисы и как меняется задержка.
//! Всё на экране построено на проверках сервисов; украшений без данных здесь нет.
use crate::components::{
    card::card,
    cards::admin_notice::admin_notice,
    install_card::install_card,
    latency_chart::{HOUR, LiveSeries, Series, has_gaps, latency_chart, live_chart},
    page_title::page_title,
    palette,
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

/// `chart_hour` — график показывает последний час по отдельным проверкам, иначе сутки.
pub fn render(
    state: &AppState,
    data: &ViewData,
    chart_hour: bool,
    cx: &mut Context<RootView>,
) -> AnyElement {
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

    // Без Zapret переключатель, график и список сервисов ничего не показывают:
    // оставляем на экране один шаг — установку.
    if !data.zapret_installed {
        let progress = data
            .logs
            .iter()
            .rev()
            .find_map(|line| line.strip_prefix("[install] "))
            .map(str::to_owned);
        return page(
            div()
                .v_flex()
                .gap_5()
                .when_some(app_update_banner(state, cx), |t, b| t.child(b))
                .child(install_card(state.zapret.installing(), progress, cx)),
        );
    }

    page(
        div()
            .v_flex()
            .gap_5()
            .when_some(app_update_banner(state, cx), |t, b| t.child(b))
            .when_some(zapret_update_banner(state, cx), |t, b| t.child(b))
            .child(header(state, data, broken, &rows, cx))
            // Причина и действие вместе: без прав обход не включится.
            .when(!data.administrator, |t| {
                t.child(admin_notice(tr("home-admin-detail"), cx))
            })
            .when(!rows.is_empty(), |t| {
                t.child(service_tiles(&rows, running, cx))
            })
            .child(chart(
                state,
                data,
                running,
                broken.is_none(),
                chart_hour,
                cx,
            )),
    )
}

/// Состояние крупной строкой, под ним строка фактов, справа переключатель и действие.
fn header(
    state: &AppState,
    data: &ViewData,
    broken: Option<&Row>,
    rows: &[Row],
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let running = data.zapret_running;
    // Без прав администратора winws не запустится.
    let admin = data.administrator;
    let error = state.zapret.last_error.as_ref().map(ToString::to_string);
    let checked_at = state
        .background
        .services
        .result
        .as_ref()
        .map(|r| r.checked_at);
    let now = data.now;
    // Во время подбора Zapret остановлен тестом: показываем ход подбора, а не «выключен».
    let picking = state.picking();
    let strategy = state
        .zapret
        .selected()
        .map(|s| {
            tr_args(
                "home-facts-strategy",
                &[("strategy", s.name.clone().into())],
            )
        })
        .unwrap_or_else(|| tr("home-facts-no-strategy"));
    let checked = match checked_at {
        Some(at) if !rows.is_empty() => tr_args("home-checked", &[("ago", ago(at, now).into())]),
        _ => tr("home-checking"),
    };

    // Строка фактов под заголовком: что происходит и на чём это основано.
    let (color, title, facts) = match (running, broken) {
        _ if picking => {
            let (done, total) = state.diagnostics.progress();
            (
                tone::warning(cx),
                tr("pick-running"),
                vec![tr_args(
                    "pick-progress",
                    &[("done", done.into()), ("total", total.into())],
                )],
            )
        }
        // Переключатель нажали, но обход не запустился. Про права говорит плашка ниже,
        // поэтому техническая причина показывается, только когда дело не в них.
        (false, _) if error.is_some() => (
            tone::error(cx),
            tr("home-bypass-failed"),
            if admin {
                vec![error.unwrap_or_default()]
            } else {
                vec![strategy]
            },
        ),
        (false, _) => (
            palette::muted(),
            tr("home-bypass-off"),
            vec![tr("home-bypass-off-hint"), strategy],
        ),
        (true, Some(row)) => (
            tone::warning(cx),
            tr_args(
                "home-service-down",
                &[("service", services::name(&row.id).to_string().into())],
            ),
            vec![tr("home-service-down-hint"), strategy],
        ),
        (true, None) => (tone::ok(cx), tr("home-bypass-on"), vec![strategy, checked]),
    };
    let can_toggle = !picking && admin;
    // Живая точка — подпись zUI: мерцает, только когда обход действительно работает.
    let pulse = if running && broken.is_none() && !picking && !cx.reduce_motion() {
        tone::pulse()
    } else {
        1.
    };

    let actions = div()
        .h_flex()
        .flex_shrink_0()
        .items_center()
        .gap_3()
        // Действие рядом с причиной: сервис не открывается — подобрать другую стратегию.
        .when(running && broken.is_some() && !picking, |d| {
            d.child(
                Button::new("home-retest")
                    .primary()
                    .label(tr("home-retest"))
                    .on_click(cx.listener(|this, _, _, cx| this.on_pick_strategy(cx))),
            )
        })
        .when(picking, |d| {
            d.child(
                Button::new("home-pick-cancel")
                    .outline()
                    .label(tr("ui-cancel"))
                    .on_click(cx.listener(|this, _, _, cx| this.on_cancel_test(cx))),
            )
        })
        .child(
            Switch::new("bypass")
                .checked(running)
                // Без прав обход не включится: действие — кнопка в плашке ниже.
                .disabled(!can_toggle)
                .accessibility_label(tr("home-bypass-switch"))
                .on_click(cx.listener(|this, _, window, cx| this.on_toggle_zapret(window, cx))),
        );

    div()
        .h_flex()
        .items_start()
        .gap_4()
        .child(
            div()
                .v_flex()
                .flex_1()
                .min_w_0()
                .gap_2()
                .child(page_title(title, None, None, cx))
                .child(
                    div()
                        .h_flex()
                        .flex_wrap()
                        .items_center()
                        .gap_x_2()
                        .gap_y_1()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(status_dot(color.opacity(pulse)))
                        .child(facts.join(" · "))
                        .child(
                            Button::new("home-change-strategy")
                                .link()
                                .small()
                                .label(tr("home-change"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.on_navigate(Tab::Strategies, cx)
                                })),
                        ),
                ),
        )
        .child(div().pt_2().child(actions))
}

/// Плитка на сервис: название, крупная задержка и состояние за сутки.
/// Полоска у названия — цвет линии этого сервиса на графике ниже.
fn service_tiles(rows: &[Row], running: bool, cx: &mut Context<RootView>) -> impl IntoElement {
    let muted = cx.theme().muted_foreground;
    let mut tiles = div().h_flex().flex_wrap().items_stretch().gap_4();
    for (index, row) in rows.iter().enumerate() {
        let connected = row.last.as_ref().map(|c| c.availability.connected());
        let (dot, detail) = match (running, connected) {
            // Обход выключен, но свежая проверка есть: видно, нужен ли он вообще.
            (false, Some(true)) => (tone::ok(cx), tr("home-service-opens")),
            (false, Some(false)) => (tone::error(cx), tr("home-service-blocked")),
            (false, None) => (palette::muted(), tr("home-service-not-checked")),
            (true, None) => (palette::muted(), tr("home-checking")),
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
        let latency = match (&row.last, !row.down) {
            (Some(check), true) if check.availability.connected() => Some(tr_args(
                "home-latency-ms",
                &[("ms", check.latency_ms.into())],
            )),
            _ => None,
        };
        tiles = tiles.child(
            card(cx)
                .flex_1()
                .min_w(px(200.))
                .gap_3()
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .text_sm()
                        .text_color(muted)
                        .child(
                            div()
                                .w_3()
                                .h(px(3.))
                                .rounded_full()
                                .bg(tone::service_color(&row.id, index, cx)),
                        )
                        .child(services::name(&row.id).to_string()),
                )
                .child(
                    div()
                        .text_size(px(30.))
                        .line_height(relative(1.))
                        .font_weight(FontWeight::MEDIUM)
                        .when(latency.is_none(), |d| d.text_color(palette::muted()))
                        .child(latency.unwrap_or_else(|| "—".into())),
                )
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .text_xs()
                        .font_family(tone::MONO)
                        .text_color(palette::muted())
                        .child(status_dot(dot))
                        .child(detail),
                ),
        );
    }
    tiles
}

/// Вариант диапазона графика: выбранный — приподнятая поверхность, как в настройках.
fn range(id: &'static str, label: String, selected: bool) -> Button {
    let button = Button::new(id).small().label(label);
    if selected {
        button
            .outline()
            .bg(palette::color(palette::ramp().raised))
            .border_color(palette::hairline_strong())
    } else {
        button.ghost()
    }
}

/// График задержки: последний час по отдельным проверкам или сутки по 15-минутным средним.
/// `healthy` — всё открывается: тогда есть смысл проверить вручную.
fn chart(
    state: &AppState,
    data: &ViewData,
    running: bool,
    healthy: bool,
    hour: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let muted = cx.theme().muted_foreground;
    let foreground = cx.theme().foreground;
    let grid = palette::hairline();
    // Ореол виден только на тёмном фоне; на светлом линии остаются просто цветными.
    let glow = cx.theme().mode.is_dark();
    let ids = &state.settings.automation.services;
    let color = |index: usize, id: &str| tone::service_color(id, index, cx);

    let legend =
        ids.iter()
            .enumerate()
            .fold(div().h_flex().flex_wrap().gap_4(), |legend, (index, id)| {
                legend.child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(div().w_3().h(px(3.)).rounded_full().bg(color(index, id)))
                        .child(services::name(id).to_string()),
                )
            });

    let empty = |text: String| {
        div()
            .h(px(148.))
            .flex()
            .items_center()
            .justify_center()
            .text_sm()
            .text_color(muted)
            .child(text)
            .into_any_element()
    };
    let (plot, gaps, axis_labels) = if hour {
        let series: Vec<LiveSeries> = ids
            .iter()
            .enumerate()
            .map(|(index, id)| LiveSeries {
                color: color(index, id),
                points: state.background.services.recent_points(id),
            })
            .collect();
        // Пока точек мало, окно короче часа: иначе первые проверки жались бы в угол.
        let oldest = series
            .iter()
            .filter_map(|s| s.points.first().map(|(time, _)| *time))
            .min()
            .unwrap_or(data.now);
        let age = data.now.saturating_sub(oldest) + 60;
        let window = (age.div_ceil(300) * 300).clamp(600, HOUR);
        let plot = if series.iter().all(|s| s.points.is_empty()) {
            empty(tr("home-chart-live-empty"))
        } else {
            live_chart(
                series,
                data.now,
                window,
                tone::warning(cx),
                grid,
                palette::muted(),
                glow,
            )
            .into_any_element()
        };
        (
            plot,
            false,
            [
                tr_args("time-ago", &[("duration", duration(window).into())]),
                tr_args("time-ago", &[("duration", duration(window / 2).into())]),
                tr("home-axis-now"),
            ],
        )
    } else {
        let series: Vec<Series> = ids
            .iter()
            .enumerate()
            .map(|(index, id)| Series {
                color: color(index, id),
                buckets: state.background.health.day(id).to_vec(),
            })
            .collect();
        let gaps = series.iter().any(|s| has_gaps(&s.buckets));
        let plot = if series.iter().all(|s| s.buckets.is_empty()) {
            // До первой 15-минутной корзины рисовать нечего: говорим, когда появится.
            empty(tr("home-chart-empty"))
        } else {
            latency_chart(
                series,
                data.now,
                running,
                tone::warning(cx),
                grid,
                palette::muted(),
                glow,
            )
            .into_any_element()
        };
        (
            plot,
            gaps,
            [
                tr("home-axis-day-ago"),
                tr("home-axis-12h"),
                tr("home-axis-now"),
            ],
        )
    };
    // Пунктир и серая полоса на графике: проверок не было.
    let legend = legend.when(gaps, |legend| {
        legend.child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child((0..3).fold(div().h_flex().gap(px(2.)), |dash, _| {
                    dash.child(div().w(px(3.)).h(px(1.)).bg(muted))
                }))
                .child(tr("home-no-data")),
        )
    });

    let busy = state.background.services.busy();
    let controls = div()
        .h_flex()
        .flex_shrink_0()
        .items_center()
        .gap_1()
        .when(running && healthy, |d| {
            d.child(
                div().pr_2().child(
                    Button::new("home-check")
                        .outline()
                        .small()
                        .label(tr("home-check-now"))
                        .loading(busy)
                        .on_click(cx.listener(|this, _, _, cx| this.on_check_services(cx))),
                ),
            )
        })
        .child(
            range("chart-hour", tr("home-range-hour"), hour)
                .on_click(cx.listener(|this, _, _, cx| this.on_chart_range(true, cx))),
        )
        .child(
            range("chart-day", tr("home-range-day"), !hour)
                .on_click(cx.listener(|this, _, _, cx| this.on_chart_range(false, cx))),
        );

    let axis = |text: String| {
        div()
            .text_xs()
            .font_family(tone::MONO)
            .text_color(palette::muted())
            .child(text)
    };
    let [left, middle, right] = axis_labels;
    card(cx)
        .gap_4()
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .gap_4()
                .text_sm()
                .text_color(muted)
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(foreground)
                        .child(tr("home-latency")),
                )
                .child(div().flex_1().min_w_0().child(legend))
                .child(controls),
        )
        .child(plot)
        .child(
            div()
                .h_flex()
                .justify_between()
                .child(axis(left))
                .child(axis(middle))
                .child(axis(right)),
        )
}
