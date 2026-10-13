//! Разметка мастера первого запуска — language.
use crate::{i18n::tr, root::RootView};
use gpui_kit::{
    component::{label::Label, *},
    *,
};
use z_core::preferences::Language;

pub(super) fn language(current: &Language, cx: &mut Context<RootView>) -> AnyElement {
    let row = crate::components::rows::language_picker::language_picker(current, "language", cx);

    div()
        .v_flex()
        .gap_3()
        .child(Label::new(tr("ui-choose-interface-language")))
        .child(row)
        .into_any_element()
}
