//! Параллельная проверка адресов; удаление JoinSet отменяет незавершённые запросы.
use crate::net::{self, VIDEO_HOST};
use anyhow::Result;
use std::{net::SocketAddr, time::Duration};

const CONNECT: Duration = Duration::from_secs(5);
const TOTAL: Duration = Duration::from_secs(8);

/// Время ответа каждого адреса или `None`, если соединение не прошло.
/// `video` — видеоузел YouTube: проверяется отдельным клиентом, который идёт на его IP.
pub(super) async fn probe(
    targets: &[String],
    video: Option<SocketAddr>,
) -> Result<Vec<(String, Option<u64>)>> {
    // Новый клиент на каждую проверку: без keep-alive от прошлой стратегии.
    let client = net::client(CONNECT, TOTAL)?;
    // Если пользователь отменил тест, JoinSet удаляется и обрывает оставшиеся запросы.
    let mut handles = tokio::task::JoinSet::new();
    for url in targets {
        let (client, url) = (client.clone(), url.clone());
        handles.spawn(async move {
            let (availability, ms) = net::probe(&client, &url).await;
            (url, availability.connected().then_some(ms))
        });
    }
    if let Some(node) = video {
        let client = net::video_client(CONNECT, TOTAL, node)?;
        handles.spawn(async move {
            let url = format!("https://{VIDEO_HOST}/generate_204");
            let (availability, ms) = net::probe(&client, &url).await;
            (video_label(node), availability.connected().then_some(ms))
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

/// Так видеоузел показывается среди адресов, которые не открылись.
pub(super) fn video_label(node: SocketAddr) -> String {
    format!("YouTube video ({})", node.ip())
}
