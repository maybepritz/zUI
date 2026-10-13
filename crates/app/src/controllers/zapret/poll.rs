//! Результаты workers и восстановление работы после установки.

use super::{ZapretController, ZapretUpdate};
use crate::controllers::job::JobPoll;

impl ZapretController {
    pub fn poll(&mut self) {
        self.is_running();

        match self.check_job.poll() {
            JobPoll::Done(Ok(Some(tag))) => {
                self.log.add(crate::i18n::tr_args(
                    "zapret-update-available",
                    &[("version", tag.clone().into())],
                ));
                self.update_state = ZapretUpdate::Available(tag);
            }
            JobPoll::Done(Ok(None)) => self.update_state = ZapretUpdate::Idle,
            JobPoll::Done(Err(e)) => {
                self.log.add(crate::i18n::tr_args(
                    "zapret-update-check-error",
                    &[("error", (e).to_string().into())],
                ));
            }
            JobPoll::Lost | JobPoll::Pending => {}
        }

        match self.job.poll() {
            JobPoll::Done(Ok(tag)) => {
                self.log.add(crate::i18n::tr_args(
                    "zapret-installed",
                    &[("version", (tag).to_string().into())],
                ));
                self.update_state = ZapretUpdate::Idle;
                self.reload_strategies();
                if std::mem::take(&mut self.restart_after) {
                    self.start();
                }
            }
            JobPoll::Done(Err(e)) => {
                self.log.add(crate::i18n::tr_args(
                    "install-error",
                    &[("error", (e).to_string().into())],
                ));
                self.update_state = ZapretUpdate::Idle;
                if std::mem::take(&mut self.restart_after) {
                    self.start();
                }
            }
            JobPoll::Lost => {
                self.log.add(crate::i18n::tr("install-worker-interrupted"));
                self.update_state = ZapretUpdate::Idle;
                self.restart_after = false;
            }
            JobPoll::Pending => {}
        }
    }
}
