//! Выбор и перечитывание стратегий.

use super::ZapretController;
use z_core::zapret::{Strategy, load_strategies};

impl ZapretController {
    pub fn selected(&self) -> Option<&Strategy> {
        self.strategies.get(self.selected_index)
    }

    pub fn select(&mut self, index: usize) -> Option<String> {
        if self.locked || self.installing() || index >= self.strategies.len() {
            return None;
        }
        let running = self.is_running();
        let previous = self.selected_index;
        self.selected_index = index;
        if running {
            self.stop();
            if !self.start() {
                self.selected_index = previous;
                self.start();
                return None;
            }
        }
        self.log.add(crate::i18n::tr_args(
            "strategy-selected",
            &[("strategy", (self.strategies[index].name).to_string().into())],
        ));
        Some(self.strategies[index].file_name.clone())
    }

    pub fn reload_strategies(&mut self) {
        let previous = self.selected().map(|s| s.file_name.clone());
        self.strategies = load_strategies(&self.runtime_dir).unwrap_or_default();
        self.selected_index = previous
            .and_then(|name| self.strategies.iter().position(|s| s.file_name == name))
            .unwrap_or(0);
        if !self.strategies.is_empty() && !self.zapret.is_running() {
            self.status = crate::i18n::text("zapret-stopped").into();
        }
    }

    pub fn index_of(&self, file_name: &str) -> Option<usize> {
        self.strategies
            .iter()
            .position(|s| s.file_name == file_name)
    }
}
