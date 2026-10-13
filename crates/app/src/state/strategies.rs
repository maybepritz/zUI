//! Выбор стратегии с сохранением настроек и статистики.

use super::AppState;
use crate::notifications::EventKind;

impl AppState {
    /// Switch through the controller before persisting the choice and statistics.
    pub fn select_strategy(&mut self, index: usize) {
        if self.profile_busy() {
            return;
        }
        let previous = self.settings.selected_strategy.clone();
        if let Some(file) = self.zapret.select(index) {
            self.settings.selected_strategy = file.clone();
            self.save_settings();
            if previous != file {
                self.background.statistics.strategy_changes = self
                    .background
                    .statistics
                    .strategy_changes
                    .saturating_add(1);
                self.event(
                    EventKind::Strategy,
                    crate::i18n::tr_args("strategy-changed", &[("strategy", file.into())]),
                );
                self.background.services.reset();
                self.persist_statistics();
            }
        }
    }

    pub fn apply_best_strategy(&mut self) {
        if let Some(ix) = self.diagnostics.best_index(&self.zapret) {
            self.select_strategy(ix);
        }
    }

    pub fn toggle_favorite(&mut self, file_name: &str) {
        let favorites = &mut self.settings.favorite_strategies;

        if let Some(ix) = favorites.iter().position(|name| name == file_name) {
            favorites.remove(ix);
        } else {
            favorites.push(file_name.to_owned());
        }

        self.save_settings();
    }
}
