//! Владение процессом и блокировка на время диагностики.

use super::ZapretController;

impl ZapretController {
    /// Stop the regular process and reserve Zapret for the diagnostics worker.
    pub fn lock(&mut self) -> bool {
        self.stop();
        if self.zapret.pid.is_some() {
            return false;
        }
        self.locked = true;
        true
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn is_running(&mut self) -> bool {
        let had_process = self.zapret.pid.is_some();
        let running = self.zapret.is_running();
        if had_process && !running {
            self.status = crate::i18n::text("zapret-exited").into();
        }
        running
    }

    /// Start the selected strategy unless installation or diagnostics owns Zapret.
    pub fn start(&mut self) -> bool {
        if self.installing() {
            self.status = crate::i18n::text("zapret-installing").into();
            return false;
        }
        if self.locked {
            self.status = crate::i18n::text("zapret-testing").into();
            return false;
        }

        let Some(strategy) = self.selected().cloned() else {
            self.status = crate::i18n::text("zapret-no-strategies").into();
            return false;
        };
        match self.zapret.start(
            &self.runtime_dir.join("bin"),
            &self.runtime_dir.join("lists"),
            &strategy.raw_args,
        ) {
            Ok(pid) => {
                self.wanted_running = true;
                self.status =
                    crate::i18n::text_args("zapret-running", &[("pid", pid.to_string().into())])
                        .into();
                true
            }
            Err(e) => {
                let message = crate::i18n::text_args(
                    "zapret-start-error",
                    &[("error", (format!("{e:#}")).to_string().into())],
                );
                self.log.add(message.to_string());
                self.status = message;
                false
            }
        }
    }

    pub fn stop(&mut self) {
        self.wanted_running = false;
        self.zapret.stop();
        self.status = if self.zapret.pid.is_some() {
            crate::i18n::text("zapret-stop-failed")
        } else {
            crate::i18n::text("zapret-stopped")
        }
        .into();
    }

    pub fn toggle(&mut self) {
        if self.is_running() {
            self.stop();
        } else {
            self.start();
        }
    }
}
