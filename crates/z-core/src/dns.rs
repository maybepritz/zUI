//! DNS state, Windows operations and presets are separated to keep recovery logic auditable.
mod manager;
mod model;
mod presets;
mod scripts;
mod system;

pub use manager::DnsManager;
pub use model::{AdapterSnapshot, DnsState, DohBackup, DohEntry};
pub use presets::{DnsPreset, presets};
pub use system::{doh_status, is_elevated, snapshot};

#[cfg(test)]
use system::{parse_json_list, parse_out, validate};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_validity() {
        let list = presets();
        assert!(!list.is_empty());
        for p in &list {
            assert!(!p.key.is_empty());
            assert!(!p.name.is_empty());
            assert!(!p.ipv4.is_empty());
            assert!(
                validate(&p.ipv4, &p.ipv6, &p.doh).is_ok(),
                "пресет {}",
                p.key
            );
        }
    }

    #[test]
    fn test_validate_rejects_garbage() {
        assert!(validate(&["1.1.1.256".into()], &[], "").is_err());
        assert!(validate(&[], &["..\\x".into()], "").is_err());
        assert!(validate(&["1.1.1.1".into()], &[], "http://x").is_err());
    }

    #[test]
    fn test_parse_json_list() {
        let single =
            r#"{"server_address":"1.1.1.1","doh_template":"https://cloudflare-dns.com/dns-query"}"#;
        let parsed_single: Vec<DohEntry> = parse_json_list(single);
        assert_eq!(parsed_single.len(), 1);
        assert_eq!(parsed_single[0].server_address, "1.1.1.1");

        let array = r#"[{"server_address":"1.1.1.1","doh_template":"https://..."},{"server_address":"1.0.0.1","doh_template":"https://..."}]"#;
        let parsed_array: Vec<DohEntry> = parse_json_list(array);
        assert_eq!(parsed_array.len(), 2);

        let parsed_empty: Vec<DohEntry> = parse_json_list("");
        assert!(parsed_empty.is_empty());
    }

    #[test]
    fn test_parse_out() {
        let out = r#"{"added":[],"modified":[],"registered":"1.1.1.1","warnings":["x"]}"#;
        let r = parse_out(out).unwrap();
        assert_eq!(r.registered, vec!["1.1.1.1".to_string()]);
        assert_eq!(r.warnings, vec!["x".to_string()]);
        assert!(parse_out("").is_err());
        assert!(parse_out("{broken}").is_err());
    }

    #[test]
    #[cfg(windows)]
    fn test_snapshot_does_not_panic() {
        let snap = snapshot();
        assert!(snap.is_ok(), "snapshot error: {:?}", snap.err());
    }
}
