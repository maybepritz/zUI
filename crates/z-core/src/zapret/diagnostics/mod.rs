//! Точка входа диагностики: модели, запуск тестов и HTTP-проверки разделены.
mod model;
mod probe;
mod runner;

pub use model::{DEFAULT_TARGETS, Options, Progress, Score};
pub use runner::{run, sort_scores};
