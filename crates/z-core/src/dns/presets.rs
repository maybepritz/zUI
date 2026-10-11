use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DnsPreset {
    pub key: String,
    pub name: String,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub doh: String,
    pub description: String,
}

fn preset(key: &str, name: &str, v4: &[&str], v6: &[&str], doh: &str, desc: &str) -> DnsPreset {
    DnsPreset {
        key: key.into(),
        name: name.into(),
        ipv4: v4.iter().map(|s| s.to_string()).collect(),
        ipv6: v6.iter().map(|s| s.to_string()).collect(),
        doh: doh.into(),
        description: desc.into(),
    }
}

pub fn presets() -> Vec<DnsPreset> {
    vec![
        preset(
            "comss",
            "Comss One DNS",
            &["83.220.169.155", "212.109.195.93"],
            &[],
            "https://dns.comss.one/dns-query",
            "Comss One DNS: обход блокировок и геоблока (сторонний сервис)",
        ),
        preset(
            "cloudflare",
            "Cloudflare",
            &["1.1.1.1", "1.0.0.1"],
            &["2606:4700:4700::1111", "2606:4700:4700::1001"],
            "https://cloudflare-dns.com/dns-query",
            "Публичный DNS. Не обходит DPI и геоблок",
        ),
        preset(
            "google",
            "Google Public DNS",
            &["8.8.8.8", "8.8.4.4"],
            &["2001:4860:4860::8888", "2001:4860:4860::8844"],
            "https://dns.google/dns-query",
            "Публичный DNS. Не обходит DPI и геоблок",
        ),
    ]
}
