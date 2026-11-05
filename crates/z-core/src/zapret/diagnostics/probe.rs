//! Параллельная проверка адресов; удаление JoinSet отменяет незавершённые запросы.
use anyhow::Result;
use std::time::Duration;

/// Время ответа каждого адреса или `None`, если соединение не прошло.
pub(super) async fn probe(targets: &[String]) -> Result<Vec<(String, Option<u64>)>> {
    // Новый клиент на каждую проверку: без keep-alive от прошлой стратегии.
    let client = crate::net::client(Duration::from_secs(5), Duration::from_secs(8))?;
    // Если пользователь отменил тест, JoinSet удаляется и обрывает оставшиеся запросы.
    let mut handles = tokio::task::JoinSet::new();
    for url in targets {
        let (client, url) = (client.clone(), url.clone());
        handles.spawn(async move {
            let (availability, ms) = crate::net::probe(&client, &url).await;
            (url, availability.connected().then_some(ms))
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
