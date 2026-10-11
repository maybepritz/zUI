use crate::{
    components::{preference::toggle, section::button_row},
    i18n::{self, tr},
    root::RootView,
};
use gpui::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{
        button::*,
        label::Label,
        progress::{Progress, ProgressCircle},
        scroll::ScrollableElement as _,
        stepper::{Stepper, StepperItem},
        *,
    },
    *,
};
use z_core::preferences::Language;

const STEPS: usize = 4;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Step {
    #[default]
    Language,
    Permissions,
    Services,
    Startup,
}

impl Step {
    fn index(self) -> usize {
        match self {
            Self::Language => 0,
            Self::Permissions => 1,
            Self::Services => 2,
            Self::Startup => 3,
        }
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Language,
            1 => Self::Permissions,
            2 => Self::Services,
            _ => Self::Startup,
        }
    }

    fn next(self) -> Self {
        Self::from_index((self.index() + 1).min(STEPS - 1))
    }
    fn previous(self) -> Self {
        Self::from_index(self.index().saturating_sub(1))
    }
}

pub fn render(view: &RootView, cx: &mut Context<RootView>) -> AnyElement {
    let state = &view.state;
    let step = view.onboarding_step;
    let idx = step.index();
    let busy = state.zapret.installing() || state.telegram.installing();

    let content = match step {
        Step::Language => language(state.settings.language, cx),
        Step::Permissions => permissions(cx),
        Step::Services => services(view, cx),
        Step::Startup => startup(view, cx),
    };

    // Степпер: нельзя перепрыгнуть вперёд, назад — можно
    let titles = [
        (IconName::Globe, tr("Язык", "Language")),
        (IconName::Info, tr("Права", "Permissions")),
        (IconName::Settings, tr("Сервисы", "Services")),
        (IconName::CircleCheck, tr("Запуск", "Startup")),
    ];
    let items = titles.into_iter().enumerate().map(|(i, (icon, title))| {
        StepperItem::new()
            .icon(icon)
            .disabled(i > idx || busy)
            .child(title)
    });

    let stepper = Stepper::new("wizard-stepper")
        .w_full()
        .selected_index(idx)
        .items(items)
        .on_click(cx.listener(|this, i: &usize, _, cx| {
            this.onboarding_step = Step::from_index(*i);
            cx.notify();
        }));

    let progress = Progress::new("wizard-progress")
        .small()
        .value((idx + 1) as f32 / STEPS as f32 * 100.0);

    let card = div()
        .v_flex()
        .gap_4()
        .p_4()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .child(content);

    let footer = div()
        .h_flex()
        .justify_between()
        .items_center()
        .child(Label::new(state.settings_status.clone()).text_sm())
        .child(
            button_row()
                .child(
                    Button::new("wizard-back")
                        .outline()
                        .label(tr("Назад", "Back"))
                        .disabled(step == Step::Language || busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.onboarding_step = this.onboarding_step.previous();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("wizard-next")
                        .primary()
                        .label(if step == Step::Startup {
                            tr("Готово", "Finish")
                        } else {
                            tr("Далее", "Next")
                        })
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.onboarding_step == Step::Startup {
                                this.state.complete_onboarding();
                            } else {
                                this.state.save_settings();
                                this.onboarding_step = this.onboarding_step.next();
                            }
                            cx.notify();
                        })),
                ),
        );

    div()
        .v_flex()
        .size_full()
        .gap_5()
        .p_6()
        .overflow_y_scrollbar()
        .child(
            div()
                .h_flex()
                .justify_between()
                .items_center()
                .child(
                    Label::new(tr("Первый запуск", "First-time setup"))
                        .text_lg()
                        .font_weight(FontWeight::BOLD),
                )
                .child(Label::new(format!("{} / {}", idx + 1, STEPS)).text_sm()),
        )
        .child(stepper)
        .child(progress)
        .child(card)
        .child(footer)
        .into_any_element()
}

fn language(current: Language, cx: &mut Context<RootView>) -> AnyElement {
    let mut row = button_row();
    for (id, name, language) in [
        ("language-ru", "Русский", Language::Ru),
        ("language-en", "English", Language::En),
    ] {
        row = row.child(
            Button::new(id)
                .label(name)
                .when(current == language, |b| b.primary())
                .when(current != language, |b| b.outline())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.state.settings.language = language;
                    i18n::set_language(language);
                    this.state.save_settings();
                    cx.notify();
                })),
        );
    }
    div()
        .v_flex()
        .gap_3()
        .child(Label::new(tr(
            "Выберите язык интерфейса",
            "Choose interface language",
        )))
        .child(row)
        .into_any_element()
}

pub fn permissions(cx: &mut Context<RootView>) -> AnyElement {
    let admin = z_core::dns::is_elevated();
    div().v_flex().gap_3()
        .child(Label::new(if admin {
            tr("Права администратора есть", "Administrator privileges available")
        } else {
            tr(
                "Без повышения прав доступны просмотр и Telegram-прокси. Zapret, DNS и hosts требуют администратора.",
                "Viewing and Telegram proxy work without elevation. Zapret, DNS and hosts require administrator privileges."
            )
        }))
        .child(Button::new("elevate-app").outline()
            .disabled(admin)
            .label(tr("Перезапустить от администратора", "Restart as administrator"))
            .on_click(cx.listener(|this, _, _, cx| {
                if this.state.elevate() {
                    cx.defer(|cx| cx.quit());
                }
                cx.notify();
            })))
        .into_any_element()
}

/// Строка сервиса: заголовок + статус слева, кнопка/спиннер справа
fn service_row(title: Label, status: Label, installing: bool, action: Button) -> Div {
    div()
        .h_flex()
        .justify_between()
        .items_center()
        .gap_3()
        .child(div().v_flex().gap_1().child(title).child(status.text_sm()))
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .when(installing, |d| {
                    d.child(ProgressCircle::new("install-spin").loading(true).size_4())
                })
                .child(action),
        )
}

fn services(view: &RootView, cx: &mut Context<RootView>) -> AnyElement {
    let state = &view.state;
    let zapret_busy = state.zapret.installing();
    let tg_busy = state.telegram.installing();

    div()
        .v_flex()
        .gap_4()
        .child(toggle(
            "wizard-zapret",
            tr("Запускать Zapret вместе с zUI", "Start Zapret with zUI"),
            state.settings.startup.zapret,
            cx,
            |s, value| s.settings.startup.zapret = value,
        ))
        .child(toggle(
            "wizard-telegram",
            tr(
                "Запускать Telegram-прокси вместе с zUI",
                "Start Telegram proxy with zUI",
            ),
            state.settings.startup.telegram,
            cx,
            |s, value| s.settings.startup.telegram = value,
        ))
        .child(service_row(
            Label::new("Zapret"),
            Label::new(state.zapret.status.clone()),
            zapret_busy,
            Button::new("wizard-install-zapret")
                .label(if state.zapret.installed() {
                    tr("Установлен", "Installed")
                } else {
                    tr("Установить", "Install")
                })
                .disabled(state.zapret.installed() || zapret_busy)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.zapret.install();
                    cx.notify();
                })),
        ))
        .child(service_row(
            Label::new(tr("Telegram-прокси", "Telegram proxy")),
            Label::new(state.telegram.message.clone()),
            tg_busy,
            Button::new("wizard-install-telegram")
                .label(if state.telegram.proxy.installed() {
                    tr("Установлен", "Installed")
                } else {
                    tr("Установить", "Install")
                })
                .disabled(state.telegram.proxy.installed() || tg_busy)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.telegram.install();
                    cx.notify();
                })),
        ))
        .into_any_element()
}

fn startup(view: &RootView, cx: &mut Context<RootView>) -> AnyElement {
    let on = view.state.settings.autostart;
    div()
        .v_flex()
        .gap_3()
        .child(Label::new(tr(
            "Запускать zUI при входе в Windows?",
            "Start zUI when you sign in to Windows?",
        )))
        .child(
            button_row()
                .child(
                    Button::new("wizard-autostart-on")
                        .label(tr("Включить", "Enable"))
                        .when(on, |b| b.primary())
                        .when(!on, |b| b.outline())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.set_autostart(true);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("wizard-autostart-off")
                        .label(tr("Выключить", "Disable"))
                        .when(!on, |b| b.primary())
                        .when(on, |b| b.outline())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.set_autostart(false);
                            cx.notify();
                        })),
                ),
        )
        .child(
            Label::new(tr(
                "Автозапуск не повышает права. Для Zapret понадобится запуск от администратора.",
                "Autostart does not elevate privileges. Zapret requires an administrator launch.",
            ))
            .text_sm(),
        )
        .into_any_element()
}
