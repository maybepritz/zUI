use super::job::{Job, JobPoll};
use crate::i18n::Text;
use std::path::PathBuf;
use z_core::{log::Log, telegram::TelegramProxy};

pub struct TelegramController {
    pub proxy: TelegramProxy,
    pub message: Text,
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
        self.message = crate::i18n::text("telegram-installing").into();
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
            Err(e) => crate::i18n::text_args(
                "telegram-error",
                &[("error", (format!("{e:#}")).to_string().into())],
            )
            .into(),
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
            Ok(()) => crate::i18n::text("telegram-media-saved").into(),
            Err(e) => crate::i18n::text_args(
                "operation-error",
                &[("error", (format!("{e:#}")).to_string().into())],
            )
            .into(),
        };
    }

    pub fn link(&mut self) -> Option<String> {
        match self.proxy.link() {
            Ok(link) => Some(link),
            Err(e) => {
                self.message = crate::i18n::text_args(
                    "telegram-link-error",
                    &[("error", (format!("{e:#}")).to_string().into())],
                )
                .into();
                None
            }
        }
    }

    pub fn poll(&mut self) {
        self.proxy.poll();
        let message = match self.job.poll() {
            JobPoll::Done(Ok(version)) => crate::i18n::text_args(
                "telegram-installed",
                &[("version", (version).to_string().into())],
            ),
            JobPoll::Done(Err(e)) => {
                crate::i18n::text_args("install-error", &[("error", (e).to_string().into())])
            }
            JobPoll::Lost => crate::i18n::text("install-interrupted"),
            JobPoll::Pending => return,
        };
        self.log.add(message.to_string());
        self.message = message.into();
    }
}
