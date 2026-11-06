//! Отвечает за первый кадр окна и передаёт подготовленное состояние обычному RootView.
mod lifecycle;
mod render;

use crate::{
    components::signal_field::SignalField,
    controllers::job::Job,
    root::RootView,
    state::{AppState, PreparationStage},
};
use gpui_kit::Entity;
use std::{
    sync::{Arc, atomic::AtomicU8},
    time::Instant,
};

pub(crate) struct StartupView {
    root: Option<Entity<RootView>>,
    worker: Job<AppState>,
    progress: Arc<AtomicU8>,
    stage: PreparationStage,
    prepared: Option<AppState>,
    update_wait_started: Option<Instant>,
    field: SignalField,
    phase: Phase,
    /// Итог проверки обновлений, который показываем перед входом в приложение.
    update_note: Option<String>,
    failed: bool,
}

/// Что показывает экран после фоновой подготовки.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Phase {
    Preparing,
    /// Итог проверки обновлений виден с этого момента.
    UpdateResult(Instant),
    /// Нет прав администратора: ждём, перезапустить или продолжить.
    AskElevation,
}
