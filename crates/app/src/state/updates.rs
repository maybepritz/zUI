//! Update scenarios coordinating mutually exclusive background operations.
use super::AppState;

impl AppState {
    /// The updater must not replace the executable while these workers are active.
    pub(crate) fn install_update(&mut self) {
        if !self.profile_busy()
            && !self.diagnostics.busy()
            && !self.zapret.installing()
            && !self.telegram.installing()
        {
            self.update.install();
        }
    }
}
