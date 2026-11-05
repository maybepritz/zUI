pub mod dns;
pub mod download;
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

/// Process creation flag that keeps console tools from opening a window.
#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;
