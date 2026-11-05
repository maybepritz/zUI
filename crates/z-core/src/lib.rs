pub mod dns;
pub mod download;
pub mod health;
pub mod log;
mod net;
pub mod paths;
pub mod preferences;
pub mod recovery;
pub mod report;
pub mod services;
pub mod settings;
pub mod statistics;
pub mod telegram;
pub mod updater;
pub mod zapret;

/// Флаг создания процесса, чтобы консольные утилиты не открывали окно.
#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;
