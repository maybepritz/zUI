//! Availability checks for the services the user picked.
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Service {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
}

const fn service(id: &'static str, name: &'static str, url: &'static str) -> Service {
    Service { id, name, url }
}

pub const SERVICES: &[Service] = &[
    //блокировка по DPI
    service("youtube", "YouTube", "https://www.youtube.com"),
    service("discord", "Discord", "https://discord.com"),
    service("instagram", "Instagram", "https://www.instagram.com"),
    service("x", "X", "https://x.com"),
    service("roblox", "Roblox", "https://www.roblox.com"),
    //геоблок DNS-обход
    service("chatgpt", "ChatGPT", "https://chatgpt.com"),
    service("claude", "Claude", "https://claude.ai"),
    service("gemini", "Gemini", "https://gemini.google.com"),
    service("copilot", "Copilot", "https://copilot.microsoft.com"),
    service("perplexity", "Perplexity", "https://www.perplexity.ai"),
    service("grok", "Grok", "https://grok.com"),
    service("mistral", "Mistral", "https://chat.mistral.ai"),
    service("spotify", "Spotify", "https://open.spotify.com"),
];

const CONTROL_URL: &str = "https://example.com";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Reachable,
    Http(u16),
    NetworkError,
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

/// Services whose id is in `ids`, in the order of `SERVICES`.
fn selected(ids: &[String]) -> impl Iterator<Item = &'static Service> + '_ {
    SERVICES.iter().filter(|s| ids.iter().any(|id| id == s.id))
}

pub fn targets(ids: &[String]) -> Vec<String> {
    selected(ids).map(|s| s.url.to_string()).collect()
}

pub fn check(ids: &[String]) -> anyhow::Result<CheckBatch> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let client = reqwest::Client::builder()
            .user_agent("zUI availability check")
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(8))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;

        let mut jobs = tokio::task::JoinSet::new();
        for service in selected(ids) {
            let client = client.clone();
            jobs.spawn(async move { probe(&client, service.id, service.url).await });
        }
        let control = probe(&client, "control", CONTROL_URL).await;

        let mut checks = Vec::new();
        while let Some(result) = jobs.join_next().await {
            checks.push(result?);
        }
        checks.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(CheckBatch {
            checks,
            internet_available: control.availability == Availability::Reachable,
            checked_at: crate::statistics::now(),
        })
    })
}

async fn probe(client: &reqwest::Client, id: &str, url: &str) -> Check {
    let started = Instant::now();
    let availability = match client.get(url).send().await {
        Ok(response) if response.status().is_success() => Availability::Reachable,
        Ok(response) => Availability::Http(response.status().as_u16()),
        Err(_) => Availability::NetworkError,
    };
    Check {
        id: id.into(),
        availability,
        latency_ms: started.elapsed().as_millis() as u64,
    }
}
