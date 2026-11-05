//! Общая основа диалогов; содержимое, действия и состояние задаёт вызывающий код.
use gpui_kit::{SharedString, component::dialog::Dialog, px};

/// Одна ширина и поведение затемнения для всех диалогов; нижнюю панель добавляет вызывающий код.
pub(crate) fn modal(dialog: Dialog, title: impl Into<SharedString>) -> Dialog {
    let title: SharedString = title.into();
    dialog
        .title(title)
        .width(px(480.))
        .overlay(true)
        .overlay_closable(false)
        .keyboard(true)
}
