use crate::{
    components::{hint::hint, preference::toggle, section::section},
    i18n::tr,
    root::RootView,
    state::AppState,
};
use gpui_kit::{
    component::{label::Label, *},
    *,
};

pub fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let p = &state.settings;

    let automation = section(tr("Автоматизация", "Automation"))
        .child(toggle(
            "watchdog",
            tr("Перезапускать winws после сбоя", "Restart winws after a crash"),
            p.automation.watchdog, cx,
            |s, value| s.settings.automation.watchdog = value,
        ))
        .child(toggle(
            "auto-switch",
            tr("Подбирать стратегию при потере доступа", "Retest strategies when access is lost"),
            p.automation.auto_switch, cx,
            |s, value| s.settings.automation.auto_switch = value,
        ))
        .child(toggle(
            "daily-retest",
            tr("Перетест раз в сутки", "Retest once a day"),
            p.automation.daily_retest, cx,
            |s, value| s.settings.automation.daily_retest = value,
        ))
        .child(hint(Label::new(tr(
            "Тест временно останавливает Zapret. Нужны работающее приложение, активный Zapret и права администратора. Проверяются выбранные нейросервисы. Без автопереключения ежедневный тест только предлагает лучшую стратегию.",
            "Testing temporarily stops Zapret. The app and Zapret must be running with administrator privileges. Tests selected AI services. With automatic switching off, daily testing only recommends a better strategy.",
        )), muted));

    let notifications = section(tr("Уведомления", "Notifications"))
        .child(toggle(
            "notify-errors",
            tr("Об ошибках", "About errors"),
            p.notifications.errors,
            cx,
            |s, value| s.settings.notifications.errors = value,
        ))
        .child(toggle(
            "notify-strategy",
            tr("О стратегиях и тестах", "About strategies and tests"),
            p.notifications.strategy,
            cx,
            |s, value| s.settings.notifications.strategy = value,
        ))
        .child(toggle(
            "notify-updates",
            tr("Об обновлениях", "About updates"),
            p.notifications.updates,
            cx,
            |s, value| s.settings.notifications.updates = value,
        ))
        .child(hint(
            Label::new(tr(
                "Уведомления приходят при свёрнутом окне.",
                "Notifications appear when the window is minimized.",
            )),
            muted,
        ));

    div()
        .v_flex()
        .gap_4()
        .child(automation)
        .child(notifications)
        .into_any_element()
}
