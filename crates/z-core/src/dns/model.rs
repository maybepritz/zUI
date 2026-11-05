use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

fn de_list<'de, D>(d: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(d)?;
    Ok(match v {
        Value::String(s) => vec![s],
        Value::Array(a) => a
            .into_iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    })
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AdapterSnapshot {
    pub interface_index: u32,
    pub interface_alias: String,
    pub interface_guid: String,
    #[serde(deserialize_with = "de_list")]
    pub ipv4: Vec<String>,
    #[serde(deserialize_with = "de_list")]
    pub ipv6: Vec<String>,
    pub ipv4_manual: bool,
    pub ipv6_manual: bool,
}

/// DoH-запись, которую добавили мы (при возврате удаляем)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DohEntry {
    pub server_address: String,
    pub doh_template: String,
}

/// Уже существовавшая DoH-запись Windows, которую мы изменили (при возврате восстанавливаем)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DohBackup {
    pub server_address: String,
    pub doh_template: String,
    pub allow_fallback: bool,
    pub auto_upgrade: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DnsState {
    pub active: bool,
    pub preset: String,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub doh: String,
    pub previous_adapters: Vec<AdapterSnapshot>,
    pub doh_added: Vec<DohEntry>,
    pub doh_modified: Vec<DohBackup>,
    /// Зарегистрирован ли DoH в Windows после последнего применения
    pub doh_active: bool,
    /// Предупреждения последней операции (показывайте их в интерфейсе)
    pub warnings: Vec<String>,
    pub last_error: String,
    pub updated_at: u64,
}

impl DnsState {
    fn knows_doh(&self, address: &str) -> bool {
        self.doh_added.iter().any(|e| e.server_address == address)
            || self
                .doh_modified
                .iter()
                .any(|e| e.server_address == address)
    }

    /// Запоминает, что было с DoH-серверами до наших изменений. Сервер, уже известный
    /// по одному из списков, пропускаем: нужно самое первое, исходное состояние.
    pub(super) fn remember_doh(&mut self, added: Vec<DohEntry>, modified: Vec<DohBackup>) {
        for entry in added {
            if !self.knows_doh(&entry.server_address) {
                self.doh_added.push(entry);
            }
        }
        for entry in modified {
            if !self.knows_doh(&entry.server_address) {
                self.doh_modified.push(entry);
            }
        }
    }
}

/// Ответ PowerShell-скриптов
#[derive(Default, Deserialize)]
#[serde(default)]
pub(super) struct PsOut {
    pub(super) added: Vec<DohEntry>,
    pub(super) modified: Vec<DohBackup>,
    #[serde(deserialize_with = "de_list")]
    pub(super) registered: Vec<String>,
    #[serde(deserialize_with = "de_list")]
    pub(super) warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn added(address: &str) -> DohEntry {
        DohEntry {
            server_address: address.into(),
            ..Default::default()
        }
    }

    fn modified(address: &str, template: &str) -> DohBackup {
        DohBackup {
            server_address: address.into(),
            doh_template: template.into(),
            ..Default::default()
        }
    }

    #[test]
    fn remember_doh_keeps_the_original_state() {
        let mut state = DnsState::default();
        state.remember_doh(vec![], vec![modified("1.1.1.1", "исходный")]);
        // Повторное применение не должно превратить изменённый сервер в добавленный:
        // иначе при возврате его удалят вместо восстановления.
        state.remember_doh(vec![added("1.1.1.1")], vec![modified("1.1.1.1", "наш")]);
        assert!(state.doh_added.is_empty());
        assert_eq!(state.doh_modified.len(), 1);
        assert_eq!(state.doh_modified[0].doh_template, "исходный");
    }
}
