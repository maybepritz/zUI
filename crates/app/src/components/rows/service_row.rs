//! Service name, status and caller-supplied action with an optional spinner.
use gpui_kit::component::progress::ProgressCircle;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{button::Button, label::Label, *},
    *,
};

/// Render a service row; the screen supplies labels, busy state and action.
pub fn service_row(title: Label, status: Label, installing: bool, action: Button) -> Div {
    div()
        .h_flex()
        .justify_between()
        .items_center()
        .gap_3()
        .child(div().v_flex().gap_1().child(title).child(status.text_sm()))
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .when(installing, |d| {
                    d.child(ProgressCircle::new("install-spin").loading(true).size_4())
                })
                .child(action),
        )
}
