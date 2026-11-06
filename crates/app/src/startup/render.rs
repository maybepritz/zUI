use super::{Phase, StartupView};
use crate::components::{signal_field::ANCHOR, tone};
use crate::{i18n::startup_tr as tr, release, state::PreparationStage};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{button::*, *},
    *,
};

/// Все шаги экрана, по которым заполняется линия под названием.
const STEPS: f32 = 6.;

impl Render for StartupView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(root) = &self.root {
            return div().size_full().child(root.clone()).into_any_element();
        }

        let (status, step) = match (self.failed, self.phase) {
            (true, _) => (tr("startup-failed"), 0.),
            (false, Phase::Preparing) => {
                let (key, step) = match self.stage {
                    PreparationStage::Settings => ("startup-settings", 1.),
                    PreparationStage::Languages => ("startup-languages", 2.),
                    PreparationStage::Components => ("startup-components", 3.),
                    PreparationStage::Updates => ("startup-updates", 4.),
                };
                (tr(key), step)
            }
            (false, Phase::UpdateResult(_)) => (self.update_note.clone().unwrap_or_default(), 5.),
            (false, Phase::AskElevation) => (tr("startup-no-admin"), STEPS),
        };
        let theme = cx.theme();
        let (ink, muted, border) = (theme.foreground, theme.muted_foreground, theme.border);
        let background = theme.background;

        // Линия под названием — индикатор: заполняется по мере шагов.
        let progress = div()
            .w(px(48.))
            .h(px(2.))
            .rounded_full()
            .bg(border)
            .overflow_hidden()
            .child(div().h_full().w(px(48. * step / STEPS)).bg(ink));

        let mut details = div()
            .v_flex()
            .items_center()
            .gap_3()
            .w_full()
            .child(div().text_sm().text_color(muted).child(status));

        if self.failed {
            details = details
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(tr("startup-failed-hint")),
                )
                .child(
                    Button::new("startup-retry")
                        .outline()
                        .label(tr("startup-retry"))
                        .on_click(cx.listener(|this, _, window, cx| this.begin(window, cx))),
                );
        } else if self.phase == Phase::AskElevation {
            details = details
                .child(
                    div()
                        .max_w(px(340.))
                        .text_center()
                        .text_sm()
                        .text_color(muted)
                        .child(tr("startup-no-admin-hint")),
                )
                .child(
                    div()
                        .h_flex()
                        .gap_2()
                        .mt_1()
                        .child(
                            Button::new("startup-elevate")
                                .primary()
                                .label(tr("ui-restart-as-administrator"))
                                .on_click(cx.listener(|this, _, _, cx| this.restart_elevated(cx))),
                        )
                        .child(
                            Button::new("startup-continue")
                                .outline()
                                .label(tr("startup-continue"))
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.open_app(window, cx)),
                                ),
                        ),
                );
        }

        let preparing = !self.failed && self.phase == Phase::Preparing;
        div()
            .id("startup-screen")
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(background)
            .text_color(ink)
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if !this.failed && !cx.reduce_motion() {
                    this.pointer_moved(event.position);
                }
            }))
            .on_hover(cx.listener(|this, hovered: &bool, _, _| {
                if !*hovered {
                    this.pointer_left();
                }
            }))
            .child(
                self.field
                    .canvas(ink, tone::ok(cx))
                    .absolute()
                    .inset_0()
                    .size_full(),
            )
            // Логотип стоит на постоянной высоте, туда же сходятся ленты фона:
            // появление вопроса о правах не сдвигает его.
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(relative(ANCHOR))
                    .px_8()
                    .v_flex()
                    .items_center()
                    .child(
                        div()
                            .v_flex()
                            .items_center()
                            .w_full()
                            .max_w(px(400.))
                            .child(img("icon.svg").size(px(80.)))
                            .child(
                                div()
                                    .mt_3()
                                    .v_flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_3xl()
                                            .font_weight(FontWeight::MEDIUM)
                                            .child("zUI"),
                                    )
                                    .child(
                                        div().text_xs().text_color(muted).child(release::label()),
                                    ),
                            )
                            .child(div().mt_5().child(progress))
                            .child(div().mt_4().w_full().child(details)),
                    ),
            )
            .when(preparing, |screen| {
                screen.child(
                    div()
                        .absolute()
                        .bottom_6()
                        .left_6()
                        .right_6()
                        .text_center()
                        .text_xs()
                        .text_color(muted)
                        .child(tr("startup-network-hint")),
                )
            })
            .into_any_element()
    }
}
