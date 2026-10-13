use super::job::{Job, JobPoll};
use crate::i18n::Text;
use std::path::Path;
use z_core::{dns::DnsManager, log::Log};

pub enum DnsAction {
    Apply(String),
    Restore,
    Reset,
}

struct DnsResult {
    manager: DnsManager,
    result: Result<(), String>,
    label: &'static str,
}

pub struct DnsController {
    pub manager: DnsManager,
    pub active_name: Text,
    pub status: Text,
    job: Job<DnsResult>,
    log: Log,
}

impl DnsController {
    pub fn new(data_dir: &Path, log: Log) -> Self {
        let manager = DnsManager::load(data_dir.join("dns_state.json"));
        Self {
            active_name: dns_name(&manager),
            status: manager.state().last_error.clone().into(),
            manager,
            job: Job::new(),
            log,
        }
    }

    pub fn busy(&self) -> bool {
        self.job.busy()
    }

    pub fn shutdown(&mut self) {
        self.job.wait();
    }

    pub fn apply(&mut self, key: &str) {
        self.begin(DnsAction::Apply(key.into()));
    }
    pub fn restore(&mut self) {
        self.begin(DnsAction::Restore);
    }
    pub fn reset(&mut self) {
        self.begin(DnsAction::Reset);
    }

    fn begin(&mut self, action: DnsAction) {
        if self.busy() {
            return;
        }
        let mut manager = self.manager.clone();
        self.status = crate::i18n::text("dns-applying").into();
        self.job.spawn(move || {
            let (result, label) = match action {
                DnsAction::Apply(key) => (manager.apply_preset(&key), "dns-applied"),
                DnsAction::Restore => (manager.restore(), "dns-restored"),
                DnsAction::Reset => (manager.reset(), "dns-reset"),
            };
            DnsResult {
                manager,
                result: result.map_err(|e| format!("{e:#}")),
                label,
            }
        });
    }

    pub fn poll(&mut self) -> Option<Result<(), String>> {
        match self.job.poll() {
            JobPoll::Done(done) => {
                let outcome = done.result.clone();
                self.manager = done.manager;
                self.active_name = dns_name(&self.manager);
                let mut message = match done.result {
                    Ok(()) => crate::i18n::text(done.label),
                    Err(e) => {
                        crate::i18n::text_args("dns-error", &[("error", (e).to_string().into())])
                    }
                };
                for w in self.manager.warnings() {
                    message.append_detail(&format!(". {w}"));
                }
                self.log.add(message.to_string());
                self.status = message.into();
                Some(outcome)
            }
            JobPoll::Lost => {
                let message = crate::i18n::text("dns-interrupted");
                self.log.add(message.to_string());
                self.status = message.into();
                Some(Err(crate::i18n::tr("dns-interrupted")))
            }
            JobPoll::Pending => None,
        }
    }
}

fn dns_name(manager: &DnsManager) -> Text {
    if !manager.is_active() {
        return crate::i18n::text("dns-system-settings").into();
    }
    z_core::dns::presets()
        .into_iter()
        .find(|p| p.key == manager.state().preset)
        .map(|p| Text::from(p.name))
        .unwrap_or_else(|| crate::i18n::text("dns-snapshot-present").into())
        .into()
}
