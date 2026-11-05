use crate::root::RootView;
use gpui_kit::{
    component::{label::Label, switch::Switch, *},
    *,
};

/// Строка настройки; изменение, сохранение и уведомление выполняет вызывающий код.
pub fn toggle(
    id: &'static str,
    title: impl Into<SharedString>,
    checked: bool,
    cx: &mut Context<RootView>,
    on_change: impl Fn(&mut RootView, bool, &mut Context<RootView>) + 'static,
) -> AnyElement {
    div()
        .h_flex()
        .justify_between()
        .items_center()
        .gap_3()
        .child(Label::new(title.into()).text_sm())
        .child(Switch::new(id).checked(checked).on_change(cx.listener(
            move |this, checked, _, cx| {
                on_change(this, *checked, cx);
            },
        )))
        .into_any_element()
}
