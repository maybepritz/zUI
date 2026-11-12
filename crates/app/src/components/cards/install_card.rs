//! Первый шаг на главной, пока zapret не установлен: одно действие на весь экран.
use super::card::card;
use crate::i18n::tr;
use crate::root::RootView;
use gpui_kit::component::{button::*, *};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

/// `progress` — последняя строка хода установки, пока она идёт.
pub fn install_card(
    installing: bool,
    progress: Option<String>,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    card(cx)
        .items_center()
        .gap_5()
        .px_6()
        .py_10()
        .child(img("icon.svg").size_12().rounded_lg())
        .child(
            div()
                .v_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::MEDIUM)
                        .child(tr("ui-zapret-is-not-installed")),
                )
                .child(
                    div()
                        .max_w(px(420.))
                        .text_sm()
                        .text_center()
                        .text_color(muted)
                        .child(tr("ui-the-latest-zapret-discord-youtube-release-will")),
                ),
        )
        .child(
            Button::new("install-zapret")
                .primary()
                .label(if installing {
                    tr("ui-installing")
                } else {
                    tr("ui-install-zapret")
                })
                .loading(installing)
                .disabled(installing)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.on_install_zapret(cx);
                })),
        )
        .when_some(progress.filter(|_| installing), |card, line| {
            card.child(div().text_xs().text_color(muted).child(line))
        })
        .into_any_element()
}
