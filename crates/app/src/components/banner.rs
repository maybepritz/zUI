use crate::i18n::tr;
use crate::root::RootView;
use gpui_kit::component::{
    button::*,
    group_box::{GroupBox, GroupBoxVariants},
    label::Label,
    *,
};
use gpui_kit::*;

pub fn banner(text: impl Into<SharedString>, actions: impl IntoElement) -> AnyElement {
    GroupBox::new()
        .outline()
        .child(
            div()
                .h_flex()
                .w_full()
                .justify_between()
                .items_center()
                .child(Label::new(text.into()).text_sm())
                .child(actions),
        )
        .into_any_element()
}

pub fn install_card(installing: bool, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    GroupBox::new()
        .outline()
        .gap_3()
        .child(
            div()
                .h_flex()
                .w_full()
                .gap_3()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_1()
                        .child(
                            Label::new(tr("Zapret не установлен", "Zapret is not installed"))
                                .text_sm()
                                .font_weight(FontWeight::BOLD),
                        )
                        .child(
                            Label::new(tr("Будет скачан последний релиз zapret-discord-youtube в папку runtime рядом с программой.", "The latest zapret-discord-youtube release will be downloaded into the runtime folder."))
                                .text_xs()
                                .text_color(muted),
                        ),
                )
                .child(
                    div().flex_shrink_0().child(
                        Button::new("install-zapret")
                            .primary()
                            .label(if installing { tr("Установка…", "Installing…") } else { tr("Установить Zapret", "Install Zapret") })
                            .disabled(installing)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.state.zapret.install();
                                cx.notify();
                            })),
                    ),
                ),
        )
        .into_any_element()
}
