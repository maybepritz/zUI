//! Parallel HTTP probes; dropping JoinSet cancels outstanding requests.
use anyhow::Result;
use std::time::{Duration, Instant};

pub(super) async fn probe(targets: &[String]) -> Result<Vec<(String, Option<u64>)>> {
    // новый клиент на каждую проверку: без keep-alive от прошлой стратегии
    let client = reqwest::Client::builder()
        .user_agent("zapret-ui/0.1")
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(8))
        .build()?;
    // Dropping JoinSet aborts outstanding probes when the user cancels the test.
    let mut handles = tokio::task::JoinSet::new();
    for url in targets {
        let (client, url) = (client.clone(), url.clone());
        handles.spawn(async move {
            let started = Instant::now();
            let ok = client
                .get(&url)
                .send()
                .await
                .is_ok_and(|response| response.status().is_success());
            (url, ok.then(|| started.elapsed().as_millis() as u64))
        });
    }
    let mut out = Vec::new();
    while let Some(result) = handles.join_next().await {
        if let Ok(r) = result {
            out.push(r);
        }
    }
    Ok(out)
}
