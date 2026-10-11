use crate::{i18n::tr, root::RootView, state::AppState};
use gpui_kit::{
    component::{label::Label, scroll::ScrollableElement as _, *},
    *,
};

mod appearance;
mod automation;
mod behavior;
mod support;

pub fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;

    let header = div()
        .h_flex()
        .justify_between()
        .items_center()
        .gap_4()
        .child(
            Label::new(tr("Настройки", "Settings"))
                .text_lg()
                .font_weight(FontWeight::BOLD),
        )
        .child(
            Label::new(state.settings_status.clone())
                .text_sm()
                .text_color(muted),
        );

    let body = div()
        .v_flex()
        .gap_4()
        .w_full()
        .max_w(px(720.))
        .mx_auto()
        .flex_shrink_0()
        .child(appearance::render(state, cx))
        .child(behavior::render(state, cx))
        .child(automation::render(state, cx))
        .child(support::render(state, cx));

    div()
        .v_flex()
        .size_full()
        .gap_4()
        .child(
            div()
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .child(header)
                .child(body),
        )
        .into_any_element()
}
