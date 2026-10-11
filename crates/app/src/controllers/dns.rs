use super::job::{Job, JobPoll};
use gpui_kit::SharedString;
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
    label: String,
}

pub struct DnsController {
    pub manager: DnsManager,
    pub active_name: SharedString,
    pub status: SharedString,
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
        self.status = "Выполняется настройка DNS…".into();
        self.job.spawn(move || {
            let (result, label) = match action {
                DnsAction::Apply(key) => (manager.apply_preset(&key), "Настройки DNS применены"),
                DnsAction::Restore => (manager.restore(), "Исходные настройки DNS восстановлены"),
                DnsAction::Reset => (manager.reset(), "DNS сброшен к DHCP"),
            };
            DnsResult {
                manager,
                result: result.map_err(|e| format!("{e:#}")),
                label: label.into(),
            }
        });
    }

    pub fn poll(&mut self) {
        match self.job.poll() {
            JobPoll::Done(done) => {
                self.manager = done.manager;
                self.active_name = dns_name(&self.manager);
                let mut message = match done.result {
                    Ok(()) => done.label,
                    Err(e) => format!("Ошибка DNS: {e}"),
                };
                for w in self.manager.warnings() {
                    message.push_str(&format!(". {w}"));
                }
                self.log.add(&message);
                self.status = message.into();
            }
            JobPoll::Lost => {
                let message = "Ошибка: задача DNS прервана. Снимок сохранён на диске.";
                self.log.add(message);
                self.status = message.into();
            }
            JobPoll::Pending => {}
        }
    }
}

fn dns_name(manager: &DnsManager) -> SharedString {
    if !manager.is_active() {
        return "Исходные настройки системы".into();
    }
    z_core::dns::presets()
        .into_iter()
        .find(|p| p.key == manager.state().preset)
        .map(|p| p.name)
        .unwrap_or_else(|| "Есть сохранённый снимок DNS".into())
        .into()
}
