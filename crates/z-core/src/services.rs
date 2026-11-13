//! Проверка доступности выбранных пользователем сервисов.
use std::time::Duration;

/// Что снимает блокировку сервиса.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    /// Блокировка по DPI: помогает стратегия zapret.
    Zapret,
    /// Геоблок: zapret не поможет, нужен DNS-обход.
    Dns,
}

#[derive(Clone, Copy)]
pub struct Service {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
    pub fix: Fix,
}

const fn service(id: &'static str, name: &'static str, url: &'static str, fix: Fix) -> Service {
    Service { id, name, url, fix }
}

pub const SERVICES: &[Service] = &[
    service("youtube", "YouTube", "https://www.youtube.com", Fix::Zapret),
    service("discord", "Discord", "https://discord.com", Fix::Zapret),
    service(
        "instagram",
        "Instagram",
        "https://www.instagram.com",
        Fix::Zapret,
    ),
    service("x", "X", "https://x.com", Fix::Zapret),
    service("roblox", "Roblox", "https://www.roblox.com", Fix::Zapret),
    service("chatgpt", "ChatGPT", "https://chatgpt.com", Fix::Dns),
    service("claude", "Claude", "https://claude.ai", Fix::Dns),
    service("gemini", "Gemini", "https://gemini.google.com", Fix::Dns),
    service(
        "copilot",
        "Copilot",
        "https://copilot.microsoft.com",
        Fix::Dns,
    ),
    service(
        "perplexity",
        "Perplexity",
        "https://www.perplexity.ai",
        Fix::Dns,
    ),
    service("grok", "Grok", "https://grok.com", Fix::Dns),
    service("mistral", "Mistral", "https://chat.mistral.ai", Fix::Dns),
    service("spotify", "Spotify", "https://open.spotify.com", Fix::Dns),
];

/// Название сервиса для интерфейса; неизвестный id возвращается как есть.
pub fn name(id: &str) -> &str {
    SERVICES.iter().find(|s| s.id == id).map_or(id, |s| s.name)
}

/// Сервисы, которые проверяются по умолчанию.
pub const DEFAULT_IDS: [&str; 2] = ["youtube", "discord"];

const CONTROL_URL: &str = "https://example.com";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Reachable,
    Http(u16),
    NetworkError,
}

impl Availability {
    /// Сервер ответил, значит соединение прошло через DPI. Код ответа (403, капча)
    /// говорит о самом сайте, а не о блокировке: по HTTPS провайдер подменить его не может.
    pub fn connected(self) -> bool {
        !matches!(self, Self::NetworkError)
    }
}

#[derive(Clone, Debug)]
pub struct Check {
    pub id: String,
    pub availability: Availability,
    pub latency_ms: u64,
}

#[derive(Clone, Debug)]
pub struct CheckBatch {
    pub checks: Vec<Check>,
    pub internet_available: bool,
    pub checked_at: u64,
}

/// Сервисы, чей id есть в `ids`, в порядке `SERVICES`.
fn selected(ids: &[String]) -> impl Iterator<Item = &'static Service> + '_ {
    SERVICES.iter().filter(|s| ids.iter().any(|id| id == s.id))
}

/// Адреса для теста стратегий: только сервисы, которые чинит zapret.
/// Сайт с геоблоком не откроется ни с одной стратегией и забракует все.
pub fn targets(ids: &[String]) -> Vec<String> {
    selected(ids)
        .filter(|s| s.fix == Fix::Zapret)
        .map(|s| s.url.to_string())
        .collect()
}

const CONNECT: Duration = Duration::from_secs(4);
const TOTAL: Duration = Duration::from_secs(8);
/// Пауза перед повтором неудачной проверки.
const RETRY_AFTER: Duration = Duration::from_secs(2);

/// Одна оборванная попытка — ещё не блокировка: браузер в таком случае тоже
/// просто повторяет запрос. Сбоем считаем, только если не открылось оба раза.
async fn probe_twice(client: &reqwest::Client, url: &str) -> (Availability, u64) {
    let first = crate::net::probe(client, url).await;
    if first.0.connected() {
        return first;
    }
    tokio::time::sleep(RETRY_AFTER).await;
    crate::net::probe(client, url).await
}

pub fn check(ids: &[String]) -> anyhow::Result<CheckBatch> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let client = crate::net::client(CONNECT, TOTAL)?;

        let mut jobs = tokio::task::JoinSet::new();
        for service in selected(ids) {
            let client = client.clone();
            jobs.spawn(async move {
                let (availability, latency_ms) = probe_twice(&client, service.url).await;
                // Задержка — время соединения с сервером, а не ответа сайта: так она
                // похожа на пинг, который пользователь видит в звонке или в браузере.
                // Если соединение принял VPN-туннель, остаётся время ответа сайта.
                let latency_ms = if availability.connected() {
                    crate::net::connect_ms(service.url, CONNECT)
                        .await
                        .unwrap_or(latency_ms)
                } else {
                    latency_ms
                };
                Check {
                    id: service.id.into(),
                    availability,
                    latency_ms,
                }
            });
        }
        let (control, _) = crate::net::probe(&client, CONTROL_URL).await;

        let mut checks = Vec::new();
        while let Some(result) = jobs.join_next().await {
            checks.push(result?);
        }
        checks.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(CheckBatch {
            checks,
            internet_available: control.connected(),
            checked_at: crate::clock::now(),
        })
    })
}
