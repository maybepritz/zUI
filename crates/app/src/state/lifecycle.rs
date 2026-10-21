//! Создание, polling и остановка сервисов приложения.

use super::background::Background;
use super::{AppState, PreparationStage};
use crate::controllers::{
    diagnostics::DiagnosticsController, dns::DnsController, telegram::TelegramController,
    update::UpdateController, zapret::ZapretController,
};
use crate::i18n::Text;
use z_core::{log::Log, settings::Settings};

impl AppState {
    /// Prepare local data off the GPUI thread. No managed services are launched here.
    pub fn prepare(mut report: impl FnMut(PreparationStage)) -> Self {
        report(PreparationStage::Settings);
        let log = Log::new();
        let mut update = UpdateController::new(log.clone());

        let runtime_dir = z_core::paths::zapret_dir();
        let data_dir = runtime_dir.parent().unwrap_or(&runtime_dir).to_path_buf();

        let (settings, recovery) = Settings::load(&data_dir.join("settings.json"));
        let settings_status = match recovery {
            None => Text::default(),
            Some(recovery) => {
                let error = format!("{:#}", recovery.error);
                log.add(format!("Settings: {error}"));
                match recovery.quarantined {
                    Some(file) => {
                        let key = if recovery.from_backup {
                            "settings-restored-from-backup"
                        } else {
                            "settings-reset-to-defaults"
                        };
                        let file = file.file_name().unwrap_or_default().to_string_lossy();
                        crate::i18n::text_args(
                            key,
                            &[("error", error.into()), ("file", file.into_owned().into())],
                        )
                    }
                    None => {
                        crate::i18n::text_args("settings-load-error", &[("error", error.into())])
                    }
                }
            }
        };

        report(PreparationStage::Languages);
        for issue in crate::i18n::initialize(&data_dir.join("locales"), &settings.language) {
            log.add(issue);
        }

        report(PreparationStage::Components);
        let mut zapret = ZapretController::new(&runtime_dir, &settings, log.clone());

        let (statistics, recovery) =
            z_core::statistics::Statistics::load(&data_dir.join("statistics.json"));
        if let Some(recovery) = recovery {
            let saved = match &recovery.quarantined {
                Some(file) => format!("damaged file saved as {}", file.display()),
                None => "damaged file could not be moved aside".into(),
            };
            let loaded = if recovery.from_backup {
                "previous copy restored"
            } else {
                "starting with empty statistics"
            };
            log.add(format!(
                "Statistics: {:#}; {saved}; {loaded}",
                recovery.error
            ));
        }
        let dns = DnsController::new(&data_dir, log.clone());
        let telegram = TelegramController::new(z_core::paths::tg_proxy_dir(), log.clone());
        report(PreparationStage::Updates);
        update.check();
        zapret.check_update();
        Self {
            profiles: Default::default(),
            background: Background::new(statistics),
            zapret,
            diagnostics: DiagnosticsController::new(log.clone()),
            dns,
            telegram,
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
