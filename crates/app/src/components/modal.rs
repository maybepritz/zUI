//! Shared dialog surface; callers own the content, actions and state.
use gpui_kit::{SharedString, component::dialog::Dialog, px};

/// One width and overlay behaviour for every dialog; the caller adds its footer.
pub(crate) fn modal(dialog: Dialog, title: impl Into<SharedString>) -> Dialog {
    let title: SharedString = title.into();
    dialog
        .title(title)
        .width(px(480.))
        .overlay(true)
        .overlay_closable(false)
        .keyboard(true)
}
