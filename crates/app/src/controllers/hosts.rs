use super::job::{Job, JobPoll};
use std::{fs, path::PathBuf};

pub struct HostsController {
    pub rules_path: PathBuf,
    backup_path: PathBuf,
    job: Job<Result<(), String>>,
    pub status: String,
}

impl HostsController {
    pub fn new(directory: &std::path::Path) -> Self {
        Self {
            rules_path: directory.join("hosts-rules.txt"),
            backup_path: directory.join("hosts-backup.json"),
            job: Job::new(),
            status: String::new()
        }
    }

    pub fn busy(&self) -> bool {
        self.job.busy()
    }

    pub fn has_backup(&self) -> bool {
        self.backup_path.is_file()
    }

    pub fn prepare_rules(&mut self) -> bool {
        let result = (|| -> std::io::Result<()> {
            use std::io::Write;
            if let Some(parent) = self.rules_path.parent() {
                fs::create_dir_all(parent)?;
            }
            match fs::OpenOptions::new().write(true).create_new(true).open(&self.rules_path) {
                Ok(mut file) => file.write_all(z_core::hosts::TEMPLATE.as_bytes()),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
                Err(error) => Err(error),
            }
        })();
        if let Err(error) = result {
            self.status = error.to_string();
            return false;
        }
        true
    }

    pub fn run(&mut self, restore: bool) {
        if self.busy() {
            return;
        }
        if !z_core::dns::is_elevated() {
            self.status = "Нужны права администратора / Administrator required".into();
            return;
        }
        let rules = self.rules_path.clone();
        let backup = self.backup_path.clone();
        self.status = "…".into();
        self.job.spawn(move || {
            let path = z_core::hosts::system_path().map_err(|e| e.to_string())?;
            if restore {
                z_core::hosts::restore(&path, &backup).map_err(|e| format!("{e:#}"))
            } else {
                let content = fs::read_to_string(rules).map_err(|e| e.to_string())?;
                z_core::hosts::apply(&path, &backup, &content).map_err(|e| format!("{e:#}"))
            }
        });
    }

    pub fn poll(&mut self) {
        match self.job.poll() {
            JobPoll::Done(Ok(())) => self.status = "Готово / Done. Перезапустите браузер / Restart your browser.".into(),
            JobPoll::Done(Err(error)) => self.status = error,
            JobPoll::Lost => self.status = "Операция прервана / Operation interrupted".into(),
            JobPoll::Pending => {}
        }
    }

    pub fn shutdown(&mut self) {
        self.job.wait();
    }
}
