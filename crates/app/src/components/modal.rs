//! Shared modal form shell; callers own form state, validation and persistence.
use crate::i18n::tr;
use gpui_kit::{
    ParentElement, SharedString,
    component::{
        button::{Button, ButtonVariants},
        dialog::{Dialog, DialogAction, DialogClose, DialogFooter},
    },
    px,
};

/// Configure a form dialog, then attach its content and `on_ok` callback.
/// Return false from `on_ok` to keep the form open after validation/save errors.
/// Create persistent input entities before calling `window.open_dialog`.
pub(crate) fn form_dialog(
    dialog: Dialog,
    title: impl Into<SharedString>,
    confirm_label: impl Into<SharedString>,
) -> Dialog {
    let title: SharedString = title.into();

    modal(dialog, title).footer(
        DialogFooter::new()
            .child(DialogClose::new().trigger(|button| button.outline().label(tr("ui-cancel"))))
            .child(
                DialogAction::new()
                    .child(Button::new("modal-confirm").primary().label(confirm_label)),
            ),
    )
}

/// Shared surface for dialogs that need custom actions instead of a form footer.
pub(crate) fn modal(dialog: Dialog, title: impl Into<SharedString>) -> Dialog {
    let title: SharedString = title.into();
    dialog
        .title(title)
        .width(px(480.))
        .overlay(true)
        .overlay_closable(false)
        .keyboard(true)
}
