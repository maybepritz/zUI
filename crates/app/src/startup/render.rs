use super::StartupView;
use crate::{i18n::startup_tr as tr, release, state::PreparationStage};
use gpui_kit::{
    component::{button::*, *},
    *,
};

impl Render for StartupView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(root) = &self.root {
            return div().size_full().child(root.clone()).into_any_element();
        }

        let status = tr(if self.failed {
            "startup-failed"
        } else {
            match self.stage {
                PreparationStage::Settings => "startup-settings",
                PreparationStage::Languages => "startup-languages",
                PreparationStage::Components => "startup-components",
                PreparationStage::Updates => "startup-updates",
            }
        });
        let mut details = div().v_flex().items_center().gap_4().w_full().child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .children((0..3).map(|index| {
                    div().size(px(3.)).rounded_full().bg(hsla(
                        0.,
                        0.,
                        0.,
                        self.particles.indicator_alpha(index),
                    ))
                }))
                .child(div().text_sm().text_color(rgb(0x626262)).child(status)),
        );

        if self.failed {
            details = details
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x888888))
                        .child(tr("startup-failed-hint")),
                )
                .child(
                    Button::new("startup-retry")
                        .outline()
                        .label(tr("startup-retry"))
                        .on_click(cx.listener(|this, _, window, cx| this.begin(window, cx))),
                );
        }

        div()
            .id("startup-screen")
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(rgb(0xffffff))
            .text_color(rgb(0x151515))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if !this.failed && !cx.reduce_motion() {
                    this.particles.pointer_moved(event.position);
                }
            }))
            .on_hover(cx.listener(|this, hovered: &bool, _, _| {
                if !*hovered {
                    this.particles.pointer_left();
                }
            }))
            .child(self.particles.canvas().absolute().inset_0().size_full())
            .child(
                div()
                    .relative()
                    .size_full()
                    .v_flex()
                    .items_center()
                    .justify_center()
                    .p_8()
                    .child(
                        div()
                            .v_flex()
                            .items_center()
                            .gap_5()
                            .w_full()
                            .max_w(px(380.))
                            .child(img("icon.svg").size(px(80.)))
                            .child(
                                div()
                                    .v_flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        div().text_3xl().font_weight(FontWeight::BOLD).child("zUI"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x888888))
                                            .child(release::label()),
                                    ),
                            )
                            .child(div().h(px(1.)).w(px(32.)).bg(rgb(0xe5e5e5)))
                            .child(details),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .bottom_6()
                    .left_6()
                    .right_6()
                    .text_center()
                    .text_xs()
                    .text_color(rgb(0x999999))
                    .child(tr("startup-network-hint")),
            )
            .into_any_element()
    }
}
