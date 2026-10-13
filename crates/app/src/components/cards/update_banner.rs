use super::banner::banner;
use crate::i18n::tr;
use crate::{
    controllers::{update::UpdateState, zapret::ZapretUpdate},
    root::RootView,
    state::AppState,
};
use gpui_kit::component::{button::*, *};
use gpui_kit::*;

fn buttons(later: Button, action: Button) -> AnyElement {
    div()
        .h_flex()
        .gap_2()
        .child(later)
        .child(action)
        .into_any_element()
}

pub fn app_update_banner(state: &AppState, cx: &mut Context<RootView>) -> Option<AnyElement> {
    Some(match &state.update.state {
        UpdateState::Available(v) => banner(
            crate::i18n::tr_args(
                "update-version-available",
                &[("version", (v).to_string().into())],
            ),
            buttons(
                Button::new("update-later")
                    .ghost()
                    .small()
                    .label(tr("ui-later"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.on_dismiss_update(cx);
                    })),
                Button::new("update-now")
                    .primary()
                    .small()
                    .label(tr("ui-update"))
                    .disabled(
                        state.diagnostics.busy()
                            || state.zapret.installing()
                            || state.telegram.installing(),
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.on_install_update(cx);
                    })),
            ),
        ),
        UpdateState::Installing => banner(tr("ui-installing-update"), div()),
        UpdateState::Done(v) => banner(
            crate::i18n::tr_args(
                "update-version-installed-restart",
                &[("version", (v).to_string().into())],
            ),
            Button::new("update-restart")
                .primary()
                .small()
                .label(tr("ui-restart"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.on_restart(cx);
                })),
        ),
        UpdateState::Failed(e) => banner(
            crate::i18n::tr_args("update-error", &[("error", (e).to_string().into())]),
            Button::new("update-dismiss")
                .ghost()
                .small()
                .label(tr("ui-close"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.on_dismiss_update(cx);
                })),
        ),
        UpdateState::Idle | UpdateState::Checking => return None,
    })
}

pub fn zapret_update_banner(state: &AppState, cx: &mut Context<RootView>) -> Option<AnyElement> {
    Some(match &state.zapret.update_state {
        ZapretUpdate::Available(tag) => banner(
            crate::i18n::tr_args(
                "zapret-update-available",
                &[("version", (tag).to_string().into())],
            ),
            buttons(
                Button::new("zapret-update-later")
                    .ghost()
                    .small()
                    .label(tr("ui-later"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.on_dismiss_zapret_update(cx);
                    })),
                Button::new("zapret-update-now")
                    .primary()
                    .small()
                    .label(tr("ui-update"))
                    .disabled(state.zapret.locked || state.zapret.installing())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.on_update_zapret(cx);
                    })),
            ),
        ),
        ZapretUpdate::Updating => banner(tr("ui-updating-zapret"), div()),
        ZapretUpdate::Idle => return None,
    })
}
