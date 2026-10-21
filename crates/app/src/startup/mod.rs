//! Owns the first window frame and hands prepared state to the regular RootView.
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
