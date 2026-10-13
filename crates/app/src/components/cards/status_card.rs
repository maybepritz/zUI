use crate::i18n::tr;
use gpui_kit::component::{
    group_box::{GroupBox, GroupBoxVariants},
    label::Label,
    *,
};
use gpui_kit::*;

pub fn status_card(
    running: bool,
    status: SharedString,
    dir: String,
    muted: Hsla,
    toggle: impl IntoElement,
) -> AnyElement {
    let color = if running {
        hsla(142.0 / 360.0, 0.76, 0.45, 1.0)
    } else {
        hsla(0.0, 0.70, 0.50, 1.0)
    };
    GroupBox::new()
        .outline()
        .gap_4()
        .child(
            div()
                .h_flex()
                .w_full()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(color)
                                        .child(if running {
                                            tr("ui-running")
                                        } else {
                                            tr("ui-stopped")
                                        }),
                                )
                                .child(Label::new(status).text_sm().text_color(muted)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted)
                                .child(crate::i18n::tr_args(
                                    "data-directory",
                                    &[("directory", (dir).to_string().into())],
                                )),
                        ),
                )
                .child(toggle),
        )
        .into_any_element()
}
