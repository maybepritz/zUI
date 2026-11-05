//! Точка входа прокси Telegram; конфиг и жизненный цикл процесса читаются отдельно.
mod config;
mod install;
mod process;
use config::asset_name;
pub use install::install;
pub use process::{ProxyStatus, TelegramProxy};

pub const RELEASES_URL: &str = "https://github.com/Flowseal/tg-ws-proxy/releases/latest";
