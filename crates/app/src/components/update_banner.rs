use crate::i18n::tr;
use super::banner::banner;
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
            format!("Доступна версия {v}"),
            buttons(
                Button::new("update-later")
                    .ghost()
                    .small()
                    .label(tr("Позже", "Later"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.state.update.dismiss();
                        cx.notify();
                    })),
                Button::new("update-now")
                    .primary()
                    .small()
                    .label(tr("Обновить", "Update"))
                    .disabled(state.diagnostics.busy() || state.zapret.installing() || state.telegram.installing() || state.hosts.busy())
                    .on_click(cx.listener(|this, _, _, cx| {
                        if !this.state.diagnostics.busy() && !this.state.zapret.installing()
                            && !this.state.telegram.installing() && !this.state.hosts.busy()
                        {
                            this.state.update.install();
                        }
                        cx.notify();
                    })),
            ),
        ),
        UpdateState::Installing => banner(tr("Установка обновления…", "Installing update…"), div()),
        UpdateState::Done(v) => banner(
            format!("Обновлено до {v}. Нужен перезапуск."),
            Button::new("update-restart")
                .primary()
                .small()
                .label(tr("Перезапустить", "Restart"))
                .on_click(cx.listener(|this, _, _, cx| {
                    if this.state.restart_app() {
                        cx.defer(|cx| cx.quit());
                    }
                })),
        ),
        UpdateState::Failed(e) => banner(
            format!("Ошибка обновления: {e}"),
            Button::new("update-dismiss")
                .ghost()
                .small()
                .label(tr("Закрыть", "Close"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.update.dismiss();
                    cx.notify();
                })),
        ),
        UpdateState::Idle | UpdateState::Checking => return None,
    })
}

pub fn zapret_update_banner(state: &AppState, cx: &mut Context<RootView>) -> Option<AnyElement> {
    Some(match &state.zapret.update_state {
        ZapretUpdate::Available(tag) => banner(
            format!("Доступна новая версия Zapret: {tag}"),
            buttons(
                Button::new("zapret-update-later")
                    .ghost()
                    .small()
                    .label(tr("Позже", "Later"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.state.zapret.dismiss_update();
                        cx.notify();
                    })),
                Button::new("zapret-update-now")
                    .primary()
                    .small()
                    .label(tr("Обновить", "Update"))
                    .disabled(state.zapret.locked || state.zapret.installing())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.state.zapret.update();
                        cx.notify();
                    })),
            ),
        ),
        ZapretUpdate::Updating => banner(tr("Обновление Zapret…", "Updating Zapret…"), div()),
        ZapretUpdate::Idle => return None,
    })
}
