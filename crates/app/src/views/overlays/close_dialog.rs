//! Close choice content; action and persistence belong to the root handler.
use crate::i18n::{Text, tr};
use gpui_kit::{
    component::{checkbox::Checkbox, label::Label, *},
    prelude::FluentBuilder as _,
    *,
};

#[derive(Default)]
pub(crate) struct CloseDialog {
    pub remember: bool,
    pub error: Option<Text>,
}
impl Render for CloseDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .w_full()
            .gap_4()
            .child(Label::new(tr("close-dialog-hint")).text_sm())
            .child(
                Checkbox::new("close-remember")
                    .label(tr("close-remember"))
                    .checked(self.remember)
                    .on_click(cx.listener(|this, checked, _, cx| {
                        this.remember = *checked;
                        cx.notify();
                    })),
            )
            .when_some(self.error.clone(), |body, error| {
                body.child(Label::new(error).text_sm().text_color(cx.theme().danger))
            })
    }
}
