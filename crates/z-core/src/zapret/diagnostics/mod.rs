//! Diagnostics facade: models, runner and HTTP probes have separate responsibilities.
mod model;
mod probe;
mod runner;

pub use model::{DEFAULT_TARGETS, Options, Progress, Score};
pub use runner::{run, sort_scores};
