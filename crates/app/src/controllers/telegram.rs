use super::job::{Job, JobPoll};
use gpui_kit::SharedString;
use std::path::PathBuf;
use z_core::{log::Log, telegram::TelegramProxy};

pub struct TelegramController {
    pub proxy: TelegramProxy,
    pub message: SharedString,
    job: Job<Result<String, String>>,
    log: Log,
}

impl TelegramController {
    pub fn new(dir: PathBuf, log: Log) -> Self {
        Self {
            proxy: TelegramProxy::new(dir),
            message: "".into(),
            job: Job::new(),
            log,
        }
    }

    pub fn installing(&self) -> bool {
        self.job.busy()
    }

    pub fn running(&mut self) -> bool {
        self.proxy.running()
    }

    pub fn install(&mut self) {
        if self.installing() || self.proxy.running() {
            return;
        }

        let dir = self.proxy.dir.clone();
        self.message = "Загрузка официального релиза и проверка SHA-256…".into();
        self.job
            .spawn(move || z_core::telegram::install(&dir).map_err(|e| format!("{e:#}")));
    }

    pub fn toggle(&mut self) {
        if self.installing() {
            return;
        }

        let result = if self.proxy.running() {
            self.proxy.stop()
        } else {
            self.proxy.start()
        };

        self.message = match result {
            Ok(()) => "".into(),
            Err(e) => format!("Ошибка TG WS Proxy: {e:#}").into(),
        };

        let line = if self.message.is_empty() {
            self.proxy.status.to_string()
        } else {
            self.message.to_string()
        };
        self.log.add(line);
    }

    pub fn fix_media(&mut self, clear: bool) {
        self.message = match self.proxy.fix_media(clear) {
            Ok(()) => "Настройка сохранена. Запустите прокси снова.".into(),
            Err(e) => format!("Ошибка: {e:#}").into(),
        };
    }

    pub fn link(&mut self) -> Option<String> {
        match self.proxy.link() {
            Ok(link) => Some(link),
            Err(e) => {
                self.message = format!("Ошибка ссылки: {e:#}").into();
                None
            }
        }
    }

    pub fn poll(&mut self) {
        self.proxy.poll();
        let message = match self.job.poll() {
            JobPoll::Done(Ok(version)) => {
                format!("Установлен TG WS Proxy {version}. Нажмите «Запустить».")
            }
            JobPoll::Done(Err(e)) => format!("Ошибка установки: {e}"),
            JobPoll::Lost => "Ошибка: установка прервана".to_string(),
            JobPoll::Pending => return,
        };
        self.log.add(&message);
        self.message = message.into();
    }
}
