//! Общий HTTP-клиент zUI и проверка, открывается ли адрес.
use crate::services::Availability;
use std::time::{Duration, Instant};

/// HTTP-клиент с User-Agent zUI и заданными таймаутами.
pub(crate) fn client(connect: Duration, total: Duration) -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("zUI/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(connect)
        .timeout(total)
        .build()
}

/// Открывает адрес и замеряет время ответа.
pub(crate) async fn probe(client: &reqwest::Client, url: &str) -> (Availability, u64) {
    let started = Instant::now();
    let availability = match client.get(url).send().await {
        Ok(response) if response.status().is_success() => Availability::Reachable,
        Ok(response) => Availability::Http(response.status().as_u16()),
        Err(_) => Availability::NetworkError,
    };
    (availability, started.elapsed().as_millis() as u64)
}
