use crate::components::{
    banner::install_card,
    config_row::{config_row, nav_button},
    log_preview::log_preview,
    status_card::status_card,
    update_banner::{app_update_banner, zapret_update_banner},
};
use crate::i18n::tr;
use crate::root::view_data::ViewData;
use crate::{
    root::{RootView, Tab},
    state::AppState,
};
use gpui_kit::component::{
    button::*,
    group_box::{GroupBox, GroupBoxVariants},
    *,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::components::page::page;

pub fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let running = data.zapret_running;
    let installed = data.zapret_installed;
    let installing = state.zapret.installing();
    let strat_name: SharedString = state
        .zapret
        .selected()
        .map(|s| s.name.clone().into())
        .unwrap_or_else(|| tr("ui-not-selected").into());
    let recent: Vec<String> = data.logs.iter().cloned().rev().take(4).rev().collect();
    let muted = cx.theme().muted_foreground;

    page(
        div()
            .v_flex()
            .gap_4()
            .when_some(app_update_banner(state, cx), |t, b| t.child(b))
            .when_some(zapret_update_banner(state, cx), |t, b| t.child(b))
            .when(!installed, |t| t.child(install_card(installing, cx)))
            .child(status_card(
                running,
                state.zapret.status.clone().into(),
                state.runtime_dir.display().to_string(),
                muted,
                Button::new("main-toggle")
                    .primary()
                    .large()
                    .disabled(installing || state.zapret.locked || state.profile_busy())
                    .label(if running {
                        tr("ui-stop-zapret")
                    } else {
                        tr("ui-start-zapret")
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.on_toggle_zapret(cx);
                    })),
            ))
            .child(
                GroupBox::new()
                    .outline()
                    .title(tr("ui-current-configuration"))
                    .gap_3()
                    .child(config_row(
                        tr("ui-strategy"),
                        strat_name,
                        nav_button(
                            "go-strategies",
                            tr("ui-choose-another"),
                            Tab::Strategies,
                            cx,
                        ),
                    ))
                    .child(config_row(
                        tr("ui-dns-server"),
                        state.dns.active_name.clone(),
                        nav_button("go-dns", tr("ui-configure-dns"), Tab::Dns, cx),
                    ))
                    .child(config_row(
                        tr("telegram-proxy-label"),
                        data.telegram_status.clone(),
                        nav_button("go-telegram", tr("ui-configure-proxy"), Tab::Telegram, cx),
                    )),
            )
            .when(crate::release::PROFILES_VISIBLE, |page| {
                page.child(crate::views::profiles::render(state, cx))
            })
            .child(log_preview(recent, cx)),
    )
}
