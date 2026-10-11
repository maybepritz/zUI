use crate::i18n::tr;
use crate::components::{
    banner::install_card,
    config_row::{config_row, nav_button},
    log_preview::log_preview,
    status_card::status_card,
    update_banner::{app_update_banner, zapret_update_banner},
};
use crate::{
    root::{RootView, Tab},
    state::AppState,
};
use gpui_kit::component::{
    button::*,
    group_box::{GroupBox, GroupBoxVariants},
    scroll::ScrollableElement as _,
    *,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

pub fn render(state: &mut AppState, cx: &mut Context<RootView>) -> AnyElement {
    let running = state.zapret.is_running();
    let installed = state.zapret.installed();
    if installed && state.zapret.strategies.is_empty() {
        state.zapret.reload_strategies();
    }
    let installing = state.zapret.installing();
    let strat_name: SharedString = state
        .zapret
        .selected()
        .map(|s| s.name.clone().into())
        .unwrap_or_else(|| tr("Не выбрана", "Not selected").into());
    let recent: Vec<String> = state.log.get().into_iter().rev().take(4).rev().collect();
    let muted = cx.theme().muted_foreground;

    div()
        .v_flex()
        .size_full()
        .gap_5()
        .overflow_y_scrollbar()
        .when_some(app_update_banner(state, cx), |t, b| t.child(b))
        .when_some(zapret_update_banner(state, cx), |t, b| t.child(b))
        .when(!installed, |t| t.child(install_card(installing, cx)))
        .child(status_card(
            running,
            state.zapret.status.clone(),
            state.runtime_dir.display().to_string(),
            muted,
            Button::new("main-toggle")
                .primary()
                .large()
                .disabled(installing || state.zapret.locked)
                .label(if running {
                    tr("Остановить Zapret", "Stop Zapret")
                } else {
                    tr("Запустить Zapret", "Start Zapret")
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.zapret.toggle();
                    cx.notify();
                })),
        ))
        .child(
            GroupBox::new()
                .outline()
                .title(tr("Текущая конфигурация", "Current configuration"))
                .gap_3()
                .child(config_row(
                    tr("Стратегия:", "Strategy:"),
                    strat_name,
                    nav_button("go-strategies", tr("Выбрать другую", "Choose another"), Tab::Strategies, cx),
                ))
                .child(config_row(
                    tr("DNS сервер:", "DNS server:"),
                    state.dns.active_name.clone(),
                    nav_button("go-dns", tr("Настроить DNS", "Configure DNS"), Tab::Dns, cx),
                ))
                .child(config_row(
                    "Telegram proxy:",
                    state.telegram.proxy.status.clone(),
                    nav_button("go-telegram", tr("Настроить proxy", "Configure proxy"), Tab::Telegram, cx),
                )),
        )
        .child(log_preview(recent, cx))
        .into_any_element()
}
