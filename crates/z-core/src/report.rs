use std::{fs, path::{Path, PathBuf}};
use anyhow::Result;
/// Keep the report human-reviewable. Proxy config, secrets and hosts contents
/// are deliberately excluded; high-risk log lines are redacted in full.

pub fn redact(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if ["secret", "token", "password", "authorization", "tg://", "cookie"]
        .iter().any(|key| lower.contains(key))
    {
        return "[redacted sensitive log entry]".into();
    }
    let mut text = line.to_string();
    for name in ["USERPROFILE", "APPDATA", "LOCALAPPDATA"] {
        if let Ok(path) = std::env::var(name) {
            if !path.is_empty() {
                text = text.replace(&path, "%USER_DATA%");
            }
        }
    }
    text
}

pub fn build(version: &str, diagnostic: &str, logs: &[String]) -> String {
    let logs = logs.iter().rev().take(250).rev().map(|s| redact(s)).collect::<Vec<_>>().join("\n");
    format!(
        "## Problem / Проблема\n\nDescribe what happened and how to reproduce it.\n\n\
        ## Environment\n\nVersion: {version}\nOS: {}\nArchitecture: {}\n\n{}\n\n\
        ## Log\n\n<details><summary>Recent events</summary>\n\n```text\n{}\n```\n</details>\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        redact(diagnostic),
        logs.replace("```", "'''")
    )
}

pub fn save(directory: &Path, report: &str) -> Result<PathBuf> {
    use std::io::Write;
    fs::create_dir_all(directory)?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let path = directory.join(format!("issue-{timestamp}-{}.md", std::process::id()));
    let mut output = fs::OpenOptions::new().write(true).create_new(true).open(&path)?;
    output.write_all(report.as_bytes())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proxy_credentials_never_enter_reports() {
        let report = build("1.0", "status: ready", &["tg://proxy?secret=abc".into(), "TOKEN=123".into()]);
        assert!(!report.contains("abc"));
        assert!(!report.contains("123"));
        assert!(report.contains("redacted"));
    }
}
