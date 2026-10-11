mod actions;
mod background;
mod testing;

use gpui_kit::SharedString;
use std::path::PathBuf;
use z_core::{log::Log, settings::Settings};

use crate::{
    controllers::{
        diagnostics::DiagnosticsController, dns::DnsController, hosts::HostsController,
        telegram::TelegramController, update::UpdateController, zapret::ZapretController,
    },
    i18n::tr,
    notifications::EventKind,
};

pub use background::Background;

pub struct AppState {
    pub log: Log,
    pub zapret: ZapretController,
    pub dns: DnsController,
    pub telegram: TelegramController,
    pub update: UpdateController,
    pub settings: Settings,
    pub settings_status: SharedString,
    pub runtime_dir: PathBuf,
    pub diagnostics: DiagnosticsController,
    pub background: Background,
    pub hosts: HostsController,
}

impl AppState {
    pub fn new() -> Self {
        let log = Log::new();

        let mut update = UpdateController::new(log.clone());
        update.check(); // фоновая проверка при старте

        let runtime_dir = z_core::paths::zapret_dir();
        let data_dir = runtime_dir.parent().unwrap_or(&runtime_dir).to_path_buf();

        let (settings, settings_status) = match Settings::load(&data_dir.join("settings.json")) {
            Ok(s) => (s, SharedString::from("")),
            Err(e) => (
                Settings::default(),
                format!("Ошибка настроек: {e:#}").into(),
            ),
        };

        let mut zapret = ZapretController::new(&runtime_dir, &settings, log.clone());
        zapret.check_update(); // фоновая проверка новой версии Zapret

        let statistics = z_core::statistics::Statistics::load(&data_dir.join("statistics.json"))
            .unwrap_or_else(|error| {
                log.add(format!("Statistics: {error:#}"));
                Default::default()
            });
        Self {
            background: Background::new(statistics),
            hosts: HostsController::new(&data_dir),
            zapret,
            diagnostics: DiagnosticsController::new(log.clone()),
            dns: DnsController::new(&data_dir, log.clone()),
            telegram: TelegramController::new(z_core::paths::tg_proxy_dir(), log.clone()),
            update,
            settings,
            settings_status,
            runtime_dir,
            log,
        }
    }

    pub fn shutdown(&mut self) {
        self.hosts.shutdown();
        self.dns.shutdown();
        self.diagnostics.shutdown();
        self.persist_statistics();
        self.zapret.stop();
        if let Err(error) = self.telegram.proxy.stop() {
            self.log
                .add(format!("Ошибка остановки TG WS Proxy: {error:#}"));
        }
    }

    pub fn restart_app(&mut self) -> bool {
        match self.update.restart() {
            Ok(()) => true,
            Err(error) => {
                self.event(EventKind::Error, error.to_string());
                false
            }
        }
    }

    pub fn save_settings(&mut self) -> bool {
        match self.settings.save(&self.data_dir().join("settings.json")) {
            Ok(()) => {
                self.settings_status = tr("Настройки сохранены", "Settings saved").into();
                true
            }
            Err(error) => {
                self.event(EventKind::Error, format!("{}: {error:#}", tr("Ошибка сохранения", "Save failed")));
                false
            }
        }
    }

    pub fn select_strategy(&mut self, index: usize) {
        let previous = self.settings.selected_strategy.clone();
        if let Some(file) = self.zapret.select(index) {
            self.settings.selected_strategy = file.clone();
            self.save_settings();
            if previous != file {
                self.background.statistics.strategy_changes =
                    self.background.statistics.strategy_changes.saturating_add(1);
                self.event(
                    EventKind::Strategy,
                    format!("{}: {file}", tr("Стратегия изменена", "Strategy changed")),
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

    pub fn poll(&mut self) {
        self.zapret.poll();
        self.dns.poll();
        self.telegram.poll();
        self.update.poll();
        if let Some(completed) = self.diagnostics.poll(&mut self.zapret) {
            self.finish_test(completed);
        }
        self.hosts.poll();
        self.poll_background();
    }
}
