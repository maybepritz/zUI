//! Создание, polling и остановка сервисов приложения.

use super::AppState;
use super::background::Background;
use crate::controllers::{
    diagnostics::DiagnosticsController, dns::DnsController, telegram::TelegramController,
    update::UpdateController, zapret::ZapretController,
};
use crate::i18n::Text;
use z_core::{log::Log, settings::Settings};

impl AppState {
    pub fn new() -> Self {
        let log = Log::new();

        let mut update = UpdateController::new(log.clone());
        update.check(); // фоновая проверка при старте

        let runtime_dir = z_core::paths::zapret_dir();
        let data_dir = runtime_dir.parent().unwrap_or(&runtime_dir).to_path_buf();

        let (settings, settings_status) = match Settings::load(&data_dir.join("settings.json")) {
            Ok(s) => (s, Text::default()),
            Err(e) => (
                Settings::default(),
                crate::i18n::text_args(
                    "settings-load-error",
                    &[("error", (format!("{e:#}")).to_string().into())],
                )
                .into(),
            ),
        };

        for issue in crate::i18n::initialize(&data_dir.join("locales"), &settings.language) {
            log.add(issue);
        }

        let mut zapret = ZapretController::new(&runtime_dir, &settings, log.clone());
        zapret.check_update(); // фоновая проверка новой версии Zapret

        let statistics = z_core::statistics::Statistics::load(&data_dir.join("statistics.json"))
            .unwrap_or_else(|error| {
                log.add(format!("Statistics: {error:#}"));
                Default::default()
            });
        Self {
            profiles: Default::default(),
            background: Background::new(statistics),
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
        if self.profile_busy() {
            self.settings.selected_profile_id = None;
            self.save_settings();
        }
        self.dns.shutdown();
        self.diagnostics.shutdown();
        self.persist_statistics();
        self.zapret.stop();
        if let Err(error) = self.telegram.proxy.stop() {
            self.log.add(crate::i18n::tr_args(
                "telegram-stop-error",
                &[("error", (format!("{error:#}")).to_string().into())],
            ));
        }
    }

    pub fn poll(&mut self) {
        self.zapret.poll();
        let dns_result = self.dns.poll();
        self.telegram.poll();
        self.update.poll();
        if let Some(completed) = self.diagnostics.poll(&mut self.zapret) {
            self.finish_test(completed);
        }
        self.poll_profile(dns_result);
        self.poll_background();
        self.reconcile_profile();
    }
}
