//! Snapshot of process/file observations used by rendering.
//!
//! Refreshed on the GPUI thread after events and polling. Rendering reads this
//! data without inspecting processes, accessing the filesystem or locking logs.
use crate::i18n::{Text, text, text_args};
use crate::state::AppState;
use z_core::telegram::ProxyStatus;
use z_core::zapret::diagnostics::Score;

#[derive(Default)]
pub(crate) struct ViewData {
    pub zapret_running: bool,
    pub zapret_installed: bool,
    pub telegram_running: bool,
    pub telegram_installed: bool,
    pub telegram_has_config: bool,
    pub telegram_has_log: bool,
    pub telegram_status: Text,
    pub administrator: bool,
    pub scores: Vec<Score>,
    pub logs: Vec<String>,
    pub now: u64,
}

impl ViewData {
    pub(super) fn read(state: &mut AppState) -> Self {
        Self {
            zapret_running: state.zapret.is_running(),
            zapret_installed: state.zapret.installed(),
            telegram_running: state.telegram.proxy.running(),
            telegram_installed: state.telegram.proxy.installed(),
            telegram_has_config: state.telegram.proxy.config_path().is_file(),
            telegram_has_log: state.telegram.proxy.log_path().is_file(),
            telegram_status: match &state.telegram.proxy.status_kind {
                ProxyStatus::Stopped => text("telegram-stopped"),
                ProxyStatus::Starting => text("telegram-starting"),
                ProxyStatus::Listening(endpoint) => text_args(
                    "telegram-listening",
                    &[("endpoint", endpoint.to_string().into())],
                ),
                ProxyStatus::Exited(status) => {
                    text_args("telegram-exited", &[("status", status.clone().into())])
                }
                ProxyStatus::ProcessError(error) => {
                    text_args("telegram-process-error", &[("error", error.clone().into())])
                }
                ProxyStatus::PortUnresponsive => text("telegram-port-unresponsive"),
            },
            administrator: z_core::dns::is_elevated(),
            scores: state.diagnostics.rows(),
            logs: state.log.get(),
            now: z_core::statistics::now(),
        }
    }
}
