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
                            Label::new(tr("ui-zapret-is-not-installed"))
                                .text_sm()
                                .font_weight(FontWeight::BOLD),
                        )
                        .child(
                            Label::new(tr("ui-the-latest-zapret-discord-youtube-release-will"))
                                .text_xs()
                                .text_color(muted),
                        ),
                )
                .child(
                    div().flex_shrink_0().child(
                        Button::new("install-zapret")
                            .primary()
                            .label(if installing {
                                tr("ui-installing")
                            } else {
                                tr("ui-install-zapret")
                            })
                            .disabled(installing)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.on_install_zapret(cx);
                            })),
                    ),
                ),
        )
        .into_any_element()
}
