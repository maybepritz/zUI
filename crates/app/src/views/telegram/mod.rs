//! Экран Telegram: состояние прокси и переключатель сверху, два шага подключения,
//! ниже — что делать, если не грузятся медиа, и служебные файлы.
use crate::components::{
    card::card,
    page::page,
    page_title::page_title,
    palette,
    section::button_row,
    tone::{self, status_dot},
};
use crate::i18n::{tr, tr_args};
use crate::root::view_data::ViewData;
use crate::{root::RootView, state::AppState};
use gpui_kit::component::{button::*, switch::Switch, *};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

pub fn render(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let installed = data.telegram_installed;
    page(
        div()
            .v_flex()
            .gap_5()
            .child(header(state, data, cx))
            .when(!installed, |column| column.child(install(state, cx)))
            .when(installed, |column| {
                column
                    .child(steps(state, data, cx))
                    .child(media(state, data, cx))
            })
            .child(files(state, data, cx)),
    )
}

/// Как на главной: название, строка состояния с точкой и переключатель справа.
fn header(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> impl IntoElement {
    let tg = &state.telegram;
    let running = data.telegram_running;
    let ready = tg.proxy.ready;
    let dot = if ready {
        tone::ok(cx)
    } else if running {
        tone::warning(cx)
    } else {
        palette::muted()
    };
    // Сообщение о последнем действии (установка, ошибка) идёт следом за состоянием.
    let message = tg.message.to_string();
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
                .child(page_title(
                    tr("tg-title"),
                    None,
                    Some(tr("tg-intro").into()),
                    cx,
                ))
                .child(
                    div()
                        .h_flex()
                        .flex_wrap()
                        .items_center()
                        .gap_2()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(status_dot(dot))
                        .child(data.telegram_status.to_string())
                        .when(!message.is_empty(), |row| {
                            row.child(div().text_color(palette::muted()).child(message))
                        }),
                ),
        )
        .child(
            div().pt_2().flex_shrink_0().child(
                Switch::new("tg-toggle")
                    .checked(running)
                    .disabled(tg.installing() || !data.telegram_installed)
                    .accessibility_label(tr("tg-switch"))
                    .on_click(cx.listener(|this, _, _, cx| this.on_toggle_telegram(cx))),
            ),
        )
}

/// Прокси ещё не установлен: на экране один шаг, как у Zapret на главной.
fn install(state: &AppState, cx: &mut Context<RootView>) -> impl IntoElement {
    let installing = state.telegram.installing();
    card(cx)
        .items_center()
        .gap_5()
        .px_6()
        .py_10()
        .child(
            div()
                .v_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::MEDIUM)
                        .child(tr("tg-install-title")),
                )
                .child(
                    div()
                        .max_w(px(420.))
                        .text_sm()
                        .text_center()
                        .text_color(cx.theme().muted_foreground)
                        .child(tr("tg-install-hint")),
                ),
        )
        .child(
            Button::new("tg-install")
                .primary()
                .label(if installing {
                    tr("ui-downloading")
                } else {
                    tr("tg-install")
                })
                .loading(installing)
                .disabled(installing)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.on_install_telegram(cx);
                })),
        )
}

/// Номер шага: выполненный шаг светлее, предстоящий приглушён.
fn step(number: &'static str, done: bool, title: String, body: impl IntoElement) -> Div {
    let ink = if done {
        palette::color(palette::ramp().text)
    } else {
        palette::muted()
    };
    div()
        .h_flex()
        .items_start()
        .gap_4()
        .child(
            div()
                .flex_shrink_0()
                .size_6()
                .rounded_full()
                .border_1()
                .border_color(if done {
                    palette::hairline_strong()
                } else {
                    palette::hairline()
                })
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .font_family(tone::MONO)
                .text_color(ink)
                .child(number),
        )
        .child(
            div()
                .v_flex()
                .flex_1()
                .min_w_0()
                .gap_3()
                .child(div().text_sm().child(title))
                .child(body),
        )
}

/// Порядок действий вместо четырёх равноправных кнопок.
fn steps(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> impl IntoElement {
    let ready = state.telegram.proxy.ready;
    card(cx)
        .gap_5()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .child(tr("tg-steps")),
        )
        .child(step("1", data.telegram_running, tr("tg-step-start"), div()))
        .child(step(
            "2",
            ready,
            tr("tg-step-connect"),
            div()
                .v_flex()
                .gap_3()
                .child(
                    button_row()
                        .child(
                            Button::new("tg-connect")
                                .primary()
                                .label(tr("ui-connect-telegram"))
                                .disabled(!ready)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.on_connect_telegram(cx);
                                })),
                        )
                        .child(
                            Button::new("tg-copy")
                                .ghost()
                                .label(tr("ui-copy-link"))
                                .disabled(!ready)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.on_copy_telegram_link(cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(palette::muted())
                        .child(tr("ui-the-port-status-confirms-that-the-local")),
                ),
        ))
}

/// Что делать, если не грузятся фото и видео: два способа и условие, при котором они доступны.
fn media(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> impl IntoElement {
    let blocked = data.telegram_running || state.telegram.installing() || !data.telegram_has_config;
    card(cx)
        .gap_3()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .child(tr("tg-media-title")),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(tr("tg-media-hint")),
        )
        .child(
            button_row()
                .child(
                    Button::new("tg-media")
                        .outline()
                        .small()
                        .label(tr("tg-media-dc4"))
                        .disabled(blocked)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_fix_telegram_media(cx);
                        })),
                )
                .child(
                    Button::new("tg-media-clear")
                        .outline()
                        .small()
                        .label(tr("tg-media-clear"))
                        .disabled(blocked)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_clear_telegram_media(cx);
                        })),
                ),
        )
}

/// Служебное: журнал, конфигурация и страница выпусков — ссылками, без карточки.
fn files(state: &AppState, data: &ViewData, cx: &mut Context<RootView>) -> impl IntoElement {
    let directory = state.telegram.proxy.dir.display().to_string();
    div()
        .v_flex()
        .gap_2()
        .child(
            button_row()
                .child(
                    Button::new("tg-log")
                        .link()
                        .small()
                        .label(tr("ui-open-log"))
                        .disabled(!data.telegram_has_log)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_open_telegram_log(cx);
                        })),
                )
                .child(
                    Button::new("tg-config")
                        .link()
                        .small()
                        .label(tr("ui-open-configuration"))
                        .disabled(data.telegram_running || !data.telegram_has_config)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_open_telegram_config(cx);
                        })),
                )
                .child(
                    Button::new("tg-releases")
                        .link()
                        .small()
                        .label(tr("ui-flowseal-releases"))
                        .on_click(cx.listener(|this, _, _, cx| this.on_telegram_releases(cx))),
                ),
        )
        .child(
            div()
                .text_xs()
                .font_family(tone::MONO)
                .text_color(palette::muted())
                .child(tr_args(
                    "data-directory",
                    &[("directory", directory.into())],
                )),
        )
}
