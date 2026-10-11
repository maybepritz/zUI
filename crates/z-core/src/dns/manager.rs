use std::{fs, path::PathBuf};
use anyhow::{Result, anyhow, bail};
use serde_json::json;
use super::{model::*, scripts::*, system::*, presets::presets};

#[derive(Clone)]
pub struct DnsManager {
    path: PathBuf,
    state: DnsState,
    load_error: Option<String>,
}

impl DnsManager {
    pub fn load(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let result = match fs::read_to_string(&path) {
            Ok(text) => {
                serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(DnsState::default()),
            Err(e) => Err(e.to_string()),
        };
        let (state, load_error) = match result {
            Ok(state) => (state, None),
            Err(e) => {
                let message = format!(
                    "Не удалось прочитать снимок DNS {}: {e}. Файл сохранён без изменений.",
                    path.display()
                );
                (
                    DnsState {
                        last_error: message.clone(),
                        ..Default::default()
                    },
                    Some(message),
                )
            }
        };
        Self {
            path,
            state,
            load_error,
        }
    }

    pub fn state(&self) -> &DnsState {
        &self.state
    }

    pub fn is_active(&self) -> bool {
        self.state.active
    }

    pub fn doh_active(&self) -> bool {
        self.state.doh_active
    }

    pub fn warnings(&self) -> &[String] {
        &self.state.warnings
    }

    fn save(&self) -> Result<()> {
        self.check_loaded()?;
        crate::settings::save_json(&self.path, &self.state)
    }

    fn check_loaded(&self) -> Result<()> {
        if let Some(error) = &self.load_error {
            bail!("{error}");
        }
        Ok(())
    }

    pub fn apply_preset(&mut self, key: &str) -> Result<()> {
        let p = presets()
            .into_iter()
            .find(|p| p.key == key)
            .ok_or_else(|| anyhow!("неизвестный пресет: {key}"))?;
        self.apply(&p.ipv4, &p.ipv6, &p.doh, &p.key)
    }

    pub fn apply(
        &mut self,
        ipv4: &[String],
        ipv6: &[String],
        doh: &str,
        label: &str,
    ) -> Result<()> {
        self.check_loaded()?;
        if !is_elevated() {
            bail!(
                "Для настройки сетевых адаптеров и DNS требуются права Администратора. Перезапустите zapret-ui от имени Администратора."
            );
        }
        if ipv4.is_empty() && ipv6.is_empty() {
            bail!("нужен хотя бы один адрес DNS");
        }
        validate(ipv4, ipv6, doh)?;

        let was_active = self.state.active;
        let previous_state = self.state.clone();
        let snap = snapshot()?;
        if !was_active || self.state.previous_adapters.is_empty() {
            // Исходный снимок берём, только пока наш DNS не активен. Иначе при
            // смене пресета мы запомнили бы собственные серверы как «исходные».
            if snap.is_empty() {
                bail!("не найдено активных сетевых адаптеров");
            }
            self.state.previous_adapters = snap;
        } else {
            // Появились новые адаптеры (например, переключились на другую сеть):
            // дописываем только их, уже сохранённые не трогаем
            for a in snap {
                if !self
                    .state
                    .previous_adapters
                    .iter()
                    .any(|p| p.interface_guid == a.interface_guid)
                {
                    self.state.previous_adapters.push(a);
                }
            }
        }

        let payload = json!({
            "adapters": self.state.previous_adapters,
            "ipv4": ipv4,
            "ipv6": ipv6,
            "doh": doh,
        });

        // Persist recovery information BEFORE changing Windows settings. If the
        // process exits during PowerShell, a later launch can still restore DNS.
        let recovery = parse_out(&run_ps(PREPARE_DOH, Some(&payload.to_string()))?)?;
        for entry in recovery.added {
            if !self
                .state
                .doh_added
                .iter()
                .any(|e| e.server_address == entry.server_address)
                && !self
                    .state
                    .doh_modified
                    .iter()
                    .any(|e| e.server_address == entry.server_address)
            {
                self.state.doh_added.push(entry);
            }
        }
        for entry in recovery.modified {
            if !self
                .state
                .doh_added
                .iter()
                .any(|e| e.server_address == entry.server_address)
                && !self
                    .state
                    .doh_modified
                    .iter()
                    .any(|e| e.server_address == entry.server_address)
            {
                self.state.doh_modified.push(entry);
            }
        }
        self.state.active = true;
        if let Err(error) = self.save() {
            self.state = previous_state;
            return Err(error);
        }

        match run_ps(APPLY, Some(&payload.to_string())) {
            Ok(out) => {
                let res = parse_out(&out)?;
                for e in res.added {
                    if !self
                        .state
                        .doh_added
                        .iter()
                        .any(|x| x.server_address == e.server_address)
                    {
                        self.state.doh_added.push(e);
                    }
                }
                for m in res.modified {
                    // нужно самое первое (исходное) значение, поэтому не перезаписываем
                    if !self
                        .state
                        .doh_modified
                        .iter()
                        .any(|x| x.server_address == m.server_address)
                        && !self
                            .state
                            .doh_added
                            .iter()
                            .any(|x| x.server_address == m.server_address)
                    {
                        self.state.doh_modified.push(m);
                    }
                }
                self.state.doh_active = !doh.is_empty() && !res.registered.is_empty();
                self.state.warnings = res.warnings;
                self.state.active = true;
                self.state.preset = label.to_string();
                self.state.ipv4 = ipv4.to_vec();
                self.state.ipv6 = ipv6.to_vec();
                self.state.doh = doh.to_string();
                self.state.last_error.clear();
                self.state.updated_at = now();
                self.save()
            }
            Err(e) => {
                if !was_active {
                    let back = json!({
                        "adapters": self.state.previous_adapters,
                        "doh_added": self.state.doh_added,
                        "doh_modified": self.state.doh_modified,
                    });
                    if run_ps(RESTORE, Some(&back.to_string())).is_ok() {
                        self.finish_inactive();
                    }
                }
                self.state.last_error = e.to_string();
                let _ = self.save();
                Err(e)
            }
        }
    }

    /// Вернуть DNS, как было до нас (ручные значения возвращаются, DHCP остаётся DHCP)
    pub fn restore(&mut self) -> Result<()> {
        self.check_loaded()?;
        if !self.state.active {
            return Ok(());
        }
        if !is_elevated() {
            bail!(
                "Для восстановления настроек DNS требуются права Администратора. Перезапустите zapret-ui от имени Администратора."
            );
        }
        if self.state.previous_adapters.is_empty() {
            bail!("нет сохранённого снимка DNS");
        }
        let payload = json!({
            "adapters": self.state.previous_adapters,
            "doh_added": self.state.doh_added,
            "doh_modified": self.state.doh_modified,
        });
        let out = match run_ps(RESTORE, Some(&payload.to_string())) {
            Ok(o) => o,
            Err(e) => {
                self.state.last_error = e.to_string();
                let _ = self.save();
                return Err(e);
            }
        };
        let res = parse_out(&out)?;
        self.finish_inactive();
        self.state.warnings = res.warnings;
        self.save()
    }

    /// Жёсткий сброс на DHCP на всех активных адаптерах.
    /// Заодно чистит остатки ручных настроек DoH в реестре от прежних версий.
    pub fn reset(&mut self) -> Result<()> {
        self.check_loaded()?;
        if !is_elevated() {
            bail!(
                "Для сброса настроек DNS требуются права Администратора. Перезапустите zapret-ui от имени Администратора."
            );
        }
        let payload = json!({
            "doh_added": self.state.doh_added,
            "doh_modified": self.state.doh_modified,
        });
        let out = match run_ps(RESET, Some(&payload.to_string())) {
            Ok(o) => o,
            Err(e) => {
                self.state.last_error = e.to_string();
                let _ = self.save();
                return Err(e);
            }
        };
        let res = parse_out(&out)?;
        self.finish_inactive();
        self.state.warnings = res.warnings;
        self.save()
    }

    fn finish_inactive(&mut self) {
        self.state.active = false;
        self.state.preset.clear();
        self.state.ipv4.clear();
        self.state.ipv6.clear();
        self.state.doh.clear();
        self.state.previous_adapters.clear();
        self.state.doh_added.clear();
        self.state.doh_modified.clear();
        self.state.doh_active = false;
        self.state.last_error.clear();
        self.state.updated_at = now();
    }
}
