//! Владение процессом и блокировка на время диагностики.

use super::ZapretController;

impl ZapretController {
    /// Останавливает обычный процесс и отдаёт Zapret задаче диагностики.
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

    /// PID работающего winws. Меняется при каждом запуске, в том числе при смене стратегии.
    pub fn pid(&self) -> Option<u32> {
        self.zapret.pid
    }

    pub fn is_running(&mut self) -> bool {
        let had_process = self.zapret.pid.is_some();
        let running = self.zapret.is_running();
        if had_process && !running {
            self.status = crate::i18n::text("zapret-exited");
            self.last_error = Some(self.status.clone());
        }
        running
    }

    /// Запускает выбранную стратегию, если Zapret не занят установкой или диагностикой.
    pub fn start(&mut self) -> bool {
        let started = self.try_start();
        // Установка и тест стратегий временно занимают Zapret, это не ошибка.
        let busy = self.locked || self.installing();
        self.last_error = (!started && !busy).then(|| self.status.clone());
        started
    }

    fn try_start(&mut self) -> bool {
        if self.installing() {
            self.status = crate::i18n::text("zapret-installing");
            return false;
        }
        if self.locked {
            self.status = crate::i18n::text("zapret-testing");
            return false;
        }

        let Some(strategy) = self.selected().cloned() else {
            self.status = crate::i18n::text("zapret-no-strategies");
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
                    crate::i18n::text_args("zapret-running", &[("pid", pid.to_string().into())]);
                true
            }
            Err(e) => {
                let message = crate::i18n::text_args(
                    "zapret-start-error",
                    &[("error", format!("{e:#}").into())],
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
        let stopped = self.zapret.pid.is_none();
        self.status = if stopped {
            crate::i18n::text("zapret-stopped")
        } else {
            crate::i18n::text("zapret-stop-failed")
        };
        self.last_error = (!stopped).then(|| self.status.clone());
    }

    pub fn toggle(&mut self) {
        if self.is_running() {
            self.stop();
        } else {
            self.start();
        }
    }
}
