//! Profile CRUD and staged application. DNS finishes before processes/preferences change.
use super::AppState;
use crate::{
    controllers::update::UpdateState,
    i18n::{self, Text},
};
use uuid::Uuid;
use z_core::profiles::{Profile, ProfileDns};

#[derive(Debug)]
pub(crate) enum ProfileError {
    EmptyName,
    LongName,
    DuplicateName,
    Busy,
    NoStrategy,
    NotFound,
    SaveFailed,
    UnknownDns,
    Privileges,
    TelegramMissing,
}

impl ProfileError {
    pub(crate) fn text(&self) -> Text {
        i18n::text(match self {
            Self::EmptyName => "profile-error-name",
            Self::LongName => "profile-error-long-name",
            Self::DuplicateName => "profile-error-duplicate",
            Self::Busy => "profile-error-busy",
            Self::NoStrategy => "profile-error-strategy",
            Self::NotFound => "profile-error-missing",
            Self::SaveFailed => "profile-error-save",
            Self::UnknownDns => "profile-error-dns",
            Self::Privileges => "profile-error-admin",
            Self::TelegramMissing => "profile-error-telegram",
        })
    }
}

#[derive(Default)]
pub(crate) struct ProfileRuntime {
    pending: Option<Profile>,
    pub status: Text,
}

impl AppState {
    pub(crate) fn profile_busy(&self) -> bool {
        self.profiles.pending.is_some()
    }

    pub(crate) fn profile_operations_busy(&self) -> bool {
        self.profile_busy()
            || self.dns.busy()
            || self.diagnostics.busy()
            || self.zapret.locked
            || self.zapret.installing()
            || self.telegram.installing()
            || matches!(self.update.state, UpdateState::Installing)
    }

    fn profile_name(&self, name: &str, except: Option<&str>) -> Result<String, ProfileError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ProfileError::EmptyName);
        }
        if name.chars().count() > 80 {
            return Err(ProfileError::LongName);
        }
        if self
            .settings
            .profiles
            .iter()
            .any(|p| Some(p.id.as_str()) != except && p.name.to_lowercase() == name.to_lowercase())
        {
            return Err(ProfileError::DuplicateName);
        }
        Ok(name.to_owned())
    }

    fn profile_snapshot(
        &self,
        id: String,
        name: String,
        dns: Option<ProfileDns>,
    ) -> Result<Profile, ProfileError> {
        if self.profile_operations_busy() {
            return Err(ProfileError::Busy);
        }
        let strategy = self
            .zapret
            .selected()
            .ok_or(ProfileError::NoStrategy)?
            .file_name
            .clone();
        let dns = dns.unwrap_or_else(|| {
            if self.dns.manager.is_active() {
                ProfileDns::Preset(self.dns.manager.state().preset.clone())
            } else {
                ProfileDns::Keep
            }
        });
        if let ProfileDns::Preset(key) = &dns {
            if !z_core::dns::presets().iter().any(|p| &p.key == key) {
                return Err(ProfileError::UnknownDns);
            }
        }
        Ok(Profile {
            id,
            name,
            strategy,
            dns,
            startup: self.settings.startup.clone(),
            automation: self.settings.automation.clone(),
        })
    }

    pub(crate) fn save_current_profile(
        &mut self,
        name: &str,
        dns: Option<ProfileDns>,
    ) -> Result<String, ProfileError> {
        let name = self.profile_name(name, None)?;
        let id = Uuid::new_v4().to_string();
        let profile = self.profile_snapshot(id.clone(), name, dns)?;
        self.settings.profiles.push(profile);
        if !self.save_settings() {
            self.settings.profiles.pop();
            return Err(ProfileError::SaveFailed);
        }
        self.profiles.status = i18n::text("profile-saved");
        Ok(id)
    }

    pub(crate) fn rename_profile(&mut self, id: &str, name: &str) -> Result<(), ProfileError> {
        if self.profile_busy() {
            return Err(ProfileError::Busy);
        }
        let name = self.profile_name(name, Some(id))?;
        let ix = self
            .settings
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or(ProfileError::NotFound)?;
        let previous = std::mem::replace(&mut self.settings.profiles[ix].name, name);
        if !self.save_settings() {
            self.settings.profiles[ix].name = previous;
            return Err(ProfileError::SaveFailed);
        }
        self.profiles.status = i18n::text("profile-renamed");
        Ok(())
    }

    pub(crate) fn update_profile(
        &mut self,
        id: &str,
        dns: Option<ProfileDns>,
    ) -> Result<(), ProfileError> {
        let ix = self
            .settings
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or(ProfileError::NotFound)?;
        let profile =
            self.profile_snapshot(id.to_owned(), self.settings.profiles[ix].name.clone(), dns)?;
        let previous = std::mem::replace(&mut self.settings.profiles[ix], profile);
        let selected = self.settings.selected_profile_id.clone();
        if selected.as_deref() == Some(id) {
            self.settings.selected_profile_id = None;
        }
        if !self.save_settings() {
            self.settings.profiles[ix] = previous;
            self.settings.selected_profile_id = selected;
            return Err(ProfileError::SaveFailed);
        }
        self.profiles.status = i18n::text("profile-updated");
        Ok(())
    }

    pub(crate) fn delete_profile(&mut self, id: &str) -> Result<(), ProfileError> {
        if self.profile_busy() {
            return Err(ProfileError::Busy);
        }
        let ix = self
            .settings
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or(ProfileError::NotFound)?;
        let previous = self.settings.profiles.remove(ix);
        let selected = self.settings.selected_profile_id.clone();
        if selected.as_deref() == Some(id) {
            self.settings.selected_profile_id = None;
        }
        if !self.save_settings() {
            self.settings.profiles.insert(ix, previous);
            self.settings.selected_profile_id = selected;
            return Err(ProfileError::SaveFailed);
        }
        self.profiles.status = i18n::text("profile-deleted");
        Ok(())
    }

    pub(crate) fn apply_profile(&mut self, id: &str) -> Result<(), ProfileError> {
        if self.profile_operations_busy() {
            return Err(ProfileError::Busy);
        }
        let profile = self
            .settings
            .profiles
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or(ProfileError::NotFound)?;
        if self.zapret.index_of(&profile.strategy).is_none() {
            return Err(ProfileError::NoStrategy);
        }
        if profile.startup.telegram && !self.telegram.proxy.installed() {
            return Err(ProfileError::TelegramMissing);
        }
        let needs_dns = match &profile.dns {
            ProfileDns::Keep => false,
            ProfileDns::Automatic => true,
            ProfileDns::Preset(key) => {
                if !z_core::dns::presets().iter().any(|p| &p.key == key) {
                    return Err(ProfileError::UnknownDns);
                }
                !self.dns.manager.is_active() || self.dns.manager.state().preset != *key
            }
        };
        if (needs_dns || profile.startup.zapret) && !z_core::dns::is_elevated() {
            return Err(ProfileError::Privileges);
        }
        self.settings.selected_profile_id = None;
        self.profiles.status =
            i18n::text_args("profile-applying", &[("name", profile.name.clone().into())]);
        if needs_dns {
            match &profile.dns {
                ProfileDns::Automatic => self.dns.reset(),
                ProfileDns::Preset(key) => self.dns.apply(key),
                ProfileDns::Keep => {}
            }
            self.profiles.pending = Some(profile);
        } else {
            self.finish_profile(profile);
        }
        Ok(())
    }

    pub(super) fn poll_profile(&mut self, dns_result: Option<Result<(), String>>) {
        let Some(result) = dns_result else {
            return;
        };
        let Some(profile) = self.profiles.pending.take() else {
            return;
        };
        match result {
            Ok(()) => self.finish_profile(profile),
            Err(error) => self.profile_failed(error),
        }
    }

    fn finish_profile(&mut self, profile: Profile) {
        let previous = self.settings.selected_strategy.clone();
        let Some(ix) = self.zapret.index_of(&profile.strategy) else {
            self.profile_failed(ProfileError::NoStrategy.text().to_string());
            return;
        };
        // Stop before selecting: select must not restart a service this profile disables.
        self.zapret.stop();
        if self.zapret.is_running() {
            self.profile_failed(self.zapret.status.to_string());
            return;
        }
        if self.zapret.select(ix).is_none() {
            self.profile_failed(ProfileError::NoStrategy.text().to_string());
            return;
        }
        self.settings.selected_strategy = profile.strategy.clone();
        if previous != profile.strategy {
            self.background.statistics.strategy_changes = self
                .background
                .statistics
                .strategy_changes
                .saturating_add(1);
            self.persist_statistics();
        }
        if profile.startup.zapret && !self.zapret.start() {
            self.profile_failed(self.zapret.status.to_string());
            return;
        }
        let telegram = if profile.startup.telegram {
            self.telegram.proxy.start()
        } else {
            self.telegram.proxy.stop()
        };
        if let Err(error) = telegram {
            self.profile_failed(format!("{error:#}"));
            return;
        }
        self.telegram.message = Default::default();
        self.settings.startup = profile.startup;
        self.settings.automation = profile.automation;
        self.background.services.reset();
        self.settings.selected_profile_id = Some(profile.id);
        if !self.save_settings() {
            self.settings.selected_profile_id = None;
            self.profile_failed(ProfileError::SaveFailed.text().to_string());
            return;
        }
        self.profiles.status = i18n::text_args("profile-applied", &[("name", profile.name.into())]);
    }

    fn profile_failed(&mut self, error: String) {
        self.settings.selected_profile_id = None;
        self.profiles.status = i18n::text_args("profile-apply-failed", &[("error", error.into())]);
        self.log.add(self.profiles.status.to_string());
        // Persist the actual partial configuration, never the requested profile marker.
        self.save_settings();
    }

    pub(crate) fn reconcile_profile(&mut self) {
        if self.profile_busy() {
            return;
        }
        let Some(id) = self.settings.selected_profile_id.clone() else {
            return;
        };
        let matches = self
            .settings
            .profiles
            .iter()
            .find(|p| p.id == id)
            .is_some_and(|p| {
                self.zapret
                    .selected()
                    .is_some_and(|s| s.file_name == p.strategy)
                    && self.settings.startup == p.startup
                    && self.settings.automation == p.automation
                    && match &p.dns {
                        ProfileDns::Keep => true,
                        ProfileDns::Automatic => !self.dns.manager.is_active(),
                        ProfileDns::Preset(key) => {
                            self.dns.manager.is_active() && self.dns.manager.state().preset == *key
                        }
                    }
            });
        let wanted = self
            .settings
            .profiles
            .iter()
            .find(|p| p.id == id)
            .map(|p| (p.startup.zapret, p.startup.telegram));
        let services_match = wanted.is_some_and(|(zapret, telegram)| {
            self.zapret.is_running() == zapret && self.telegram.running() == telegram
        });
        if !matches || !services_match {
            self.settings.selected_profile_id = None;
            self.profiles.status = i18n::text("profile-modified");
            self.save_settings();
        }
    }
}
