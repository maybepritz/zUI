use crate::root::{RootView, Tab};
use gpui_kit::component::{button::*, label::Label, *};
use gpui_kit::*;

pub fn config_row(
    label: &'static str,
    value: impl Into<SharedString>,
    button: Button,
) -> impl IntoElement {
    div()
        .h_flex()
        .w_full()
        .justify_between()
        .items_center()
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(Label::new(label).text_sm().font_weight(FontWeight::BOLD))
                .child(Label::new(value.into()).text_sm()),
        )
        .child(button)
}

/// Кнопка перехода на вкладку.
pub fn nav_button(
    id: &'static str,
    label: &'static str,
    tab: Tab,
    cx: &mut Context<RootView>,
) -> Button {
    Button::new(id)
        .outline()
        .small()
        .label(label)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.tab = tab;
            cx.notify();
        }))
}
