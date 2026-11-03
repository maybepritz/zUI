use crate::components::tone::{self, status_dot};
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
    toggle: impl IntoElement,
    cx: &App,
) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    // A stopped service is not an error, so it gets the neutral dot.
    let dot = if running { tone::ok(cx) } else { muted };
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
                                .child(status_dot(dot))
                                .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(
                                    if running {
                                        tr("ui-running")
                                    } else {
                                        tr("ui-stopped")
                                    },
                                ))
                                .child(Label::new(status).text_sm().text_color(muted)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted)
                                .child(crate::i18n::tr_args(
                                    "data-directory",
                                    &[("directory", dir.to_string().into())],
                                )),
                        ),
                )
                .child(toggle),
        )
        .into_any_element()
}
