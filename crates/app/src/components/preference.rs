use crate::{root::RootView, state::AppState};
use gpui_kit::{component::{label::Label, switch::Switch, *}, *};

pub fn toggle(
    id: &'static str,
    title: &'static str,
    checked: bool,
    cx: &mut Context<RootView>,
    update: impl Fn(&mut AppState, bool) + 'static,
) -> AnyElement {
    div().h_flex().justify_between().items_center().gap_3()
        .child(Label::new(title).text_sm())
        .child(Switch::new(id).checked(checked)
            .on_change(cx.listener(move |this, checked, _, cx| {
                update(&mut this.state, *checked);
                this.state.save_settings();
                cx.notify();
            })))
        .into_any_element()
}
