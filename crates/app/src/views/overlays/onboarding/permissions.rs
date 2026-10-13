//! Разметка мастера первого запуска — permissions.
use crate::{i18n::tr, root::RootView};
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub(super) fn permissions(admin: bool, cx: &mut Context<RootView>) -> AnyElement {
    div()
        .v_flex()
        .gap_3()
        .child(Label::new(if admin {
            tr("ui-administrator-privileges-available")
        } else {
            tr("ui-viewing-and-telegram-proxy-work-without-elevation")
        }))
        .child(
            Button::new("elevate-app")
                .outline()
                .disabled(admin)
                .label(tr("ui-restart-as-administrator"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.on_elevate(cx);
                })),
        )
        .into_any_element()
}
