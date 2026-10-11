use std::{process::Command, time::{SystemTime, UNIX_EPOCH}};
use anyhow::{Result, anyhow, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::Value;
use super::{model::*, scripts::*};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub(super) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(windows)]
pub fn is_elevated() -> bool {
    #[link(name = "shell32")]
    unsafe extern "system" {
        fn IsUserAnAdmin() -> i32;
    }
    unsafe { IsUserAnAdmin() != 0 }
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    true
}

pub(super) fn validate(ipv4: &[String], ipv6: &[String], doh: &str) -> Result<()> {
    for s in ipv4 {
        if s.parse::<std::net::Ipv4Addr>().is_err() {
            bail!("некорректный IPv4: {s}");
        }
    }
    for s in ipv6 {
        if s.parse::<std::net::Ipv6Addr>().is_err() {
            bail!("некорректный IPv6: {s}");
        }
    }
    if !doh.is_empty() && (!doh.starts_with("https://") || doh.contains(char::is_whitespace)) {
        bail!("DoH-адрес должен быть вида https://...");
    }
    Ok(())
}

fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_start_matches('\u{feff}')
        .trim()
        .to_string()
}

fn last_json(out: &str) -> &str {
    out.lines()
        .rev()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with('[') || t.starts_with('{')
        })
        .unwrap_or("")
}

pub(super) fn run_ps(body: &str, payload: Option<&str>) -> Result<String> {
    let script = format!(
        "{PREAMBLE}{HELPERS}try {{ {body} }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
    );
    // -EncodedCommand: base64 от UTF-16LE, так не нужно ничего экранировать
    let utf16: Vec<u8> = script
        .encode_utf16()
        .flat_map(|u| u.to_le_bytes())
        .collect();
    let encoded = STANDARD.encode(utf16);
    if encoded.len() > 30_000 {
        bail!("скрипт PowerShell слишком длинный для командной строки");
    }

    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-OutputFormat",
        "Text",
        "-EncodedCommand",
        &encoded,
    ]);
    if let Some(p) = payload {
        // через переменную окружения, чтобы не возиться с кавычками и кириллицей
        cmd.env("ZAPRET_DNS_PAYLOAD", p);
    }
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW

    let out = cmd.output()?;
    let stdout = decode(&out.stdout);
    let stderr = decode(&out.stderr);
    if !out.status.success() {
        bail!("{}", if stderr.is_empty() { stdout } else { stderr });
    }
    Ok(stdout)
}

pub(super) fn parse_json_list<T: serde::de::DeserializeOwned>(json: &str) -> Vec<T> {
    if json.trim().is_empty() {
        return Vec::new();
    }
    match serde_json::from_str::<Value>(json) {
        Ok(Value::Array(a)) => a
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
        Ok(obj @ Value::Object(_)) => serde_json::from_value(obj).ok().into_iter().collect(),
        _ => Vec::new(),
    }
}

pub(super) fn parse_out(out: &str) -> Result<PsOut> {
    let json = last_json(out);
    if json.is_empty() {
        bail!("PowerShell не вернул результат; снимок DNS сохранён");
    }
    serde_json::from_str(json).map_err(|e| anyhow!("Не удалось разобрать ответ PowerShell: {e}"))
}

// ───────────────────────── API ─────────────────────────

/// Текущие DNS активных физических адаптеров (VPN/TUN отфильтрованы)
pub fn snapshot() -> Result<Vec<AdapterSnapshot>> {
    let out = run_ps(SNAPSHOT, None)?;
    let rows: Vec<AdapterSnapshot> = parse_json_list(last_json(&out));
    Ok(rows
        .into_iter()
        .filter(|r| r.interface_index > 0 && !r.interface_alias.is_empty())
        .collect())
}

/// DoH-серверы, зарегистрированные в Windows (пусто, если DoH не поддерживается)
pub fn doh_status() -> Result<Vec<DohEntry>> {
    let out = run_ps(DOH_LIST, None)?;
    Ok(parse_json_list(last_json(&out)))
}
