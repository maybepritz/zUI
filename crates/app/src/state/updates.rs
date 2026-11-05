//! Сценарии обновления, согласующие взаимоисключающие фоновые операции.
use super::AppState;

impl AppState {
    /// Обновление не должно заменять exe, пока работают эти задачи.
    pub(crate) fn install_update(&mut self) {
        if !self.diagnostics.busy() && !self.zapret.installing() && !self.telegram.installing() {
            self.update.install();
        }
    }
}
