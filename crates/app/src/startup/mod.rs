//! Отвечает за первый кадр окна и передаёт подготовленное состояние обычному RootView.
mod lifecycle;
mod render;

use crate::{
    components::particle_background::ParticleField,
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
    particles: ParticleField,
    failed: bool,
}
