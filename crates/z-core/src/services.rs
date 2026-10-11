use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Service {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
}

pub const SERVICES: &[Service] = &[
    Service {
        id: "chatgpt",
        name: "ChatGPT",
        url: "https://chatgpt.com"
    },
    Service {
        id: "claude",
        name: "Claude",
        url: "https://claude.ai"
    },
    Service {
        id: "gemini",
        name: "Gemini",
        url: "https://gemini.google.com"
    },
    Service {
        id: "copilot",
        name: "Copilot",
        url: "https://copilot.microsoft.com"
    },
    Service {
        id: "perplexity",
        name: "Perplexity",
        url: "https://www.perplexity.ai"
    },
];

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

pub fn targets(ids: &[String]) -> Vec<String> {
    SERVICES.iter().filter(|s| ids.iter().any(|id| id == s.id))
        .map(|s| s.url.to_string()).collect()
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
        for service in SERVICES.iter().filter(|s| ids.iter().any(|id| id == s.id)) {
            let client = client.clone();
            jobs.spawn(async move {
                probe(&client, service.id, service.url).await
            });
        }
        let internet = probe(&client, "control", "https://example.com").await;
        let mut checks = Vec::new();
        while let Some(result) = jobs.join_next().await {
            checks.push(result?);
        }
        checks.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(CheckBatch {
            checks,
            internet_available: internet.availability == Availability::Reachable,
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
        latency_ms: started.elapsed().as_millis() as u64
    }
}
