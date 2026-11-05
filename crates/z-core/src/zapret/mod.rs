//! Интеграция с Zapret; реэкспорты сохраняют публичный API при перестановке внутренних модулей.
pub mod diagnostics;
pub mod install;
mod model;
mod process;
mod strategies;
mod strategy_parser;

pub use model::Strategy;
pub use process::Zapret;
pub use strategies::{ensure_user_lists, load_strategies};
