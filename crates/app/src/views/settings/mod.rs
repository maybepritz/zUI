//! Настройки: сверху то, что меняют чаще всего, остальное свёрнуто в «Дополнительно»,
//! служебное собрано в «Помощь». Все настройки из файла остаются доступными.
use crate::state::{CloseBehavior, Preference};
use crate::{
    components::{
        hint::hint,
        page::page,
        palette,
        preference::toggle,
        rows::language_picker::language_picker,
        section::{button_row, section},
    },
    i18n::{tr, tr_args},
    root::RootView,
    state::AppState,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{button::*, label::Label, switch::Switch, *},
    *,
};
use z_core::preferences::ThemePreference;

pub fn render(state: &AppState, advanced_open: bool, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;

    let header = div()
        .h_flex()
        .justify_between()
        .items_center()
        .gap_4()
        .child(
            div()
                .text_size(px(30.))
                .line_height(relative(1.15))
                .child(tr("ui-settings")),
        )
        .child(
            Label::new(state.settings_status.clone())
                .text_sm()
                .text_color(muted),
        );

    let more = Button::new("settings-more")
        .ghost()
        .icon(if advanced_open {
            IconName::ChevronDown
        } else {
            IconName::ChevronRight
        })
        .label(tr("settings-more"))
        .on_click(cx.listener(|this, _, _, cx| this.on_toggle_advanced_settings(cx)));

    page(
        div()
            .v_flex()
            .gap_6()
            .child(header)
            .child(main(state, cx))
            .child(
                div()
                    .v_flex()
                    .gap_3()
                    .child(div().h_flex().child(more))
                    .when(advanced_open, |column| column.child(advanced(state, cx))),
            )
            .child(help(state, cx)),
    )
}

/// Вариант в ряду выбора: выбранный — приподнятая поверхность с заметной границей,
/// а не главная кнопка, иначе на экране было бы несколько «главных» действий.
fn option(id: &'static str, label: String, selected: bool) -> Button {
    let button = Button::new(id).label(label);
    if selected {
        button
            .outline()
            .bg(palette::color(palette::ramp().raised))
            .border_color(palette::hairline_strong())
    } else {
        button.ghost()
    }
}

/// Строка настройки: название слева, управление справа; узкому окну разрешён перенос.
fn row(title: String, control: impl IntoElement) -> Div {
    div()
        .h_flex()
        .flex_wrap()
        .justify_between()
        .items_center()
        .gap_3()
        .min_h_8()
        .child(Label::new(title).text_sm())
        .child(control)
}

/// То, что меняют чаще всего: автозапуск, вид, закрытие окна и уведомления.
fn main(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let settings = &state.settings;

    let mut themes = button_row();
    for (id, name, preference) in [
        ("theme-system", tr("ui-system"), ThemePreference::System),
        ("theme-light", tr("ui-light"), ThemePreference::Light),
        ("theme-dark", tr("ui-dark"), ThemePreference::Dark),
    ] {
        themes = themes.child(option(id, name, settings.theme == preference).on_click(
            cx.listener(move |this, _, window, cx| this.on_theme(preference, window, cx)),
        ));
    }

    let current = CloseBehavior::of(settings);
    let mut closing = button_row();
    for (id, key, behavior) in [
        ("close-ask", "settings-close-ask", CloseBehavior::Ask),
        ("close-tray", "settings-close-tray", CloseBehavior::Tray),
        ("close-exit", "settings-close-exit", CloseBehavior::Exit),
    ] {
        closing = closing.child(
            option(id, tr(key), current == behavior)
                .on_click(cx.listener(move |this, _, _, cx| this.on_close_behavior(behavior, cx))),
        );
    }

    let notifications = &settings.notifications;
    let translators = crate::i18n::translators(&settings.language).join(", ");

    section(tr("settings-main"))
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(row(
                    tr("settings-start-with-windows"),
                    Switch::new("start-with-windows")
                        .checked(settings.autostart)
                        .on_change(cx.listener(|this, checked, _, cx| {
                            this.on_start_with_windows(*checked, cx);
                        })),
                ))
                .child(hint(
                    Label::new(tr(
                        "ui-autostart-does-not-elevate-privileges-zapret-requires",
                    )),
                    muted,
                )),
        )
        .child(row(tr("ui-theme"), themes))
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(row(
                    tr("ui-interface-language"),
                    language_picker(&settings.language, "settings", cx),
                ))
                .when(!translators.is_empty(), |column| {
                    column.child(
                        Label::new(tr_args("locale-credit", &[("names", translators.into())]))
                            .text_xs()
                            .text_color(muted),
                    )
                }),
        )
        .child(row(tr("settings-on-close"), closing))
        .child(row(
            tr("ui-notifications"),
            Switch::new("notifications")
                .checked(notifications.errors || notifications.strategy || notifications.updates)
                .on_change(cx.listener(|this, checked, _, cx| {
                    this.on_notifications(*checked, cx);
                })),
        ))
        .into_any_element()
}

/// Редкие настройки: что запускать, как восстанавливаться и о чём именно уведомлять.
fn advanced(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let settings = &state.settings;
    let switch = |id, key, value, preference, cx: &mut Context<RootView>| {
        toggle(id, tr(key), value, cx, move |this, value, cx| {
            this.on_preference(preference, value, cx)
        })
    };
    let group = |key: &str| {
        div()
            .pt_2()
            .text_xs()
            .text_color(palette::muted())
            .child(tr(key))
    };

    section("")
        .child(group("ui-startup"))
        .child(switch(
            "startup-zapret",
            "ui-start-zapret-with-zui",
            settings.startup.zapret,
            Preference::StartupZapret,
            cx,
        ))
        .child(switch(
            "startup-telegram",
            "ui-start-telegram-proxy-with-zui",
            settings.startup.telegram,
            Preference::StartupTelegram,
            cx,
        ))
        .child(switch(
            "ask-elevation",
            "settings-ask-elevation",
            settings.ask_elevation,
            Preference::AskElevation,
            cx,
        ))
        .child(group("ui-automation"))
        .child(switch(
            "watchdog",
            "ui-restart-winws-after-a-crash",
            settings.automation.watchdog,
            Preference::Watchdog,
            cx,
        ))
        .child(switch(
            "auto-switch",
            "ui-retest-strategies-when-access-is-lost",
            settings.automation.auto_switch,
            Preference::AutoSwitch,
            cx,
        ))
        .child(switch(
            "daily-retest",
            "ui-retest-once-a-day",
            settings.automation.daily_retest,
            Preference::DailyRetest,
            cx,
        ))
        .child(hint(
            Label::new(tr("ui-testing-temporarily-stops-zapret-the-app-and")),
            muted,
        ))
        .child(group("ui-notifications"))
        .child(switch(
            "notify-errors",
            "ui-about-errors",
            settings.notifications.errors,
            Preference::NotifyErrors,
            cx,
        ))
        .child(switch(
            "notify-strategy",
            "ui-about-strategies-and-tests",
            settings.notifications.strategy,
            Preference::NotifyStrategy,
            cx,
        ))
        .child(switch(
            "notify-updates",
            "ui-about-updates",
            settings.notifications.updates,
            Preference::NotifyUpdates,
            cx,
        ))
        .child(
            div().h_flex().pt_2().child(
                Button::new("reopen-onboarding")
                    .outline()
                    .small()
                    .label(tr("ui-open-setup-wizard"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.on_reopen_onboarding(cx);
                    })),
            ),
        )
        .into_any_element()
}

/// Версия, обновления, отчёт о проблеме и выход: нужно редко, поэтому одной тихой группой.
fn help(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let version = format!(
        "{} {}",
        state.update.current_version(),
        crate::release::codename()
    );
    let action = |id: &'static str, key: &str| Button::new(id).outline().small().label(tr(key));

    section(tr("settings-help"))
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .justify_between()
                .items_center()
                .gap_3()
                .child(
                    Label::new(tr_args("version-label", &[("version", version.into())])).text_sm(),
                )
                .child(
                    button_row()
                        .child(action("check-update", "ui-check-for-updates").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.on_check_update(cx);
                            }),
                        ))
                        .child(
                            action("show-whats-new", "ui-what-s-new").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.on_show_whats_new(cx);
                                },
                            )),
                        ),
                ),
        )
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(Label::new(tr("ui-report-a-problem")).text_sm())
                .child(hint(
                    Label::new(tr("ui-the-report-includes-the-version-checks-statistics")),
                    muted,
                ))
                .child(
                    button_row()
                        .child(
                            action("export-report", "ui-save-report").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.on_export_report(cx);
                                },
                            )),
                        )
                        .child(
                            action("copy-report", "ui-copy-for-github").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.on_copy_report(cx);
                                },
                            )),
                        )
                        .child(
                            Button::new("open-issue")
                                .outline()
                                .small()
                                .label("GitHub issue")
                                .on_click(cx.listener(|this, _, _, cx| this.on_open_issue(cx))),
                        ),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .justify_between()
                .gap_3()
                .child(
                    action("open-runtime", "ui-open-data-folder")
                        .on_click(cx.listener(|this, _, _, cx| this.on_open_runtime(cx))),
                )
                .child(
                    Button::new("quit-app")
                        .danger()
                        .small()
                        .label(tr("ui-exit-application"))
                        .on_click(cx.listener(|this, _, _, cx| this.on_quit(cx))),
                ),
        )
        .into_any_element()
}
