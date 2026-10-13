use crate::state::Preference;
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

    let automation = section(tr("ui-automation"))
        .child(toggle(
            "watchdog",
            tr("ui-restart-winws-after-a-crash"),
            p.automation.watchdog,
            cx,
            |this, value, cx| this.on_preference(Preference::Watchdog, value, cx),
        ))
        .child(toggle(
            "auto-switch",
            tr("ui-retest-strategies-when-access-is-lost"),
            p.automation.auto_switch,
            cx,
            |this, value, cx| this.on_preference(Preference::AutoSwitch, value, cx),
        ))
        .child(toggle(
            "daily-retest",
            tr("ui-retest-once-a-day"),
            p.automation.daily_retest,
            cx,
            |this, value, cx| this.on_preference(Preference::DailyRetest, value, cx),
        ))
        .child(hint(
            Label::new(tr("ui-testing-temporarily-stops-zapret-the-app-and")),
            muted,
        ));

    let notifications = section(tr("ui-notifications"))
        .child(toggle(
            "notify-errors",
            tr("ui-about-errors"),
            p.notifications.errors,
            cx,
            |this, value, cx| this.on_preference(Preference::NotifyErrors, value, cx),
        ))
        .child(toggle(
            "notify-strategy",
            tr("ui-about-strategies-and-tests"),
            p.notifications.strategy,
            cx,
            |this, value, cx| this.on_preference(Preference::NotifyStrategy, value, cx),
        ))
        .child(toggle(
            "notify-updates",
            tr("ui-about-updates"),
            p.notifications.updates,
            cx,
            |this, value, cx| this.on_preference(Preference::NotifyUpdates, value, cx),
        ))
        .child(hint(
            Label::new(tr("ui-notifications-appear-when-the-window-is-minimized")),
            muted,
        ));

    div()
        .v_flex()
        .gap_4()
        .child(automation)
        .child(notifications)
        .into_any_element()
}
