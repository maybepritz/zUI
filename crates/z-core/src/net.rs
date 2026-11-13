//! Общий HTTP-клиент zUI и проверка, открывается ли адрес.
use crate::services::Availability;
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

/// Столько ответа нужно прочитать, чтобы адрес считался открытым. DPI часто
/// пропускает начало соединения и обрывает его на 16–20 КБ: сайт «открылся»,
/// а видео и картинки висят.
const BODY_BYTES: usize = 32 * 1024;

/// Адрес, через который проверяется видеоузел YouTube. Сертификат
/// `*.googlevideo.com` подходит к нему, а соединение идёт на IP узла.
pub const VIDEO_HOST: &str = "redirector.googlevideo.com";

fn builder(connect: Duration, total: Duration) -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .user_agent(concat!("zUI/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(connect)
        .timeout(total)
}

/// HTTP-клиент с User-Agent zUI и заданными таймаутами.
pub(crate) fn client(connect: Duration, total: Duration) -> reqwest::Result<reqwest::Client> {
    builder(connect, total).build()
}

/// Клиент, который ходит на `VIDEO_HOST` по адресу `node`, а не по DNS.
pub(crate) fn video_client(
    connect: Duration,
    total: Duration,
    node: SocketAddr,
) -> reqwest::Result<reqwest::Client> {
    builder(connect, total).resolve(VIDEO_HOST, node).build()
}

/// Открывает адрес и замеряет время до ответа сервера. Открытым адрес
/// считается, только если удалось прочитать начало ответа (см. `BODY_BYTES`).
pub(crate) async fn probe(client: &reqwest::Client, url: &str) -> (Availability, u64) {
    let started = Instant::now();
    let mut response = match client.get(url).send().await {
        Ok(response) => response,
        Err(_) => {
            return (
                Availability::NetworkError,
                started.elapsed().as_millis() as u64,
            );
        }
    };
    let ms = started.elapsed().as_millis() as u64;
    let status = response.status();
    let mut read = 0;
    while read < BODY_BYTES {
        match response.chunk().await {
            Ok(Some(chunk)) => read += chunk.len(),
            Ok(None) => break,
            Err(_) => return (Availability::NetworkError, ms),
        }
    }
    let availability = if status.is_success() {
        Availability::Reachable
    } else {
        Availability::Http(status.as_u16())
    };
    (availability, ms)
}

/// Время установки TCP-соединения с сервером адреса — почти чистый сетевой пинг.
/// В отличие от [`probe`], сюда не входят TLS, обход и работа веб-сервера, поэтому
/// цифра сравнима с пингом в звонке или в браузере. `None`, если не удалось соединиться.
pub(crate) async fn connect_ms(url: &str, timeout: Duration) -> Option<u64> {
    let url = reqwest::Url::parse(url).ok()?;
    let host = url.host_str()?;
    let port = url.port_or_known_default()?;
    // Адрес узнаём заранее: время DNS к пингу не относится.
    let address = tokio::net::lookup_host((host, port)).await.ok()?.next()?;
    let started = Instant::now();
    tokio::time::timeout(timeout, tokio::net::TcpStream::connect(address))
        .await
        .ok()?
        .ok()?;
    let ms = started.elapsed().as_millis() as u64;
    // VPN-туннель (sing-box, Clash, WireGuard) принимает соединение прямо на компьютере:
    // выходит 0–1 мс, а до сервера на самом деле далеко. Такой замер ничего не говорит.
    (ms >= LOCAL_CONNECT_MS).then_some(ms)
}

/// Быстрее этого соединение с удалённым сервером не устанавливается: значит,
/// его принял кто-то на этом компьютере.
const LOCAL_CONNECT_MS: u64 = 2;

/// Ближайший к пользователю видеоузел YouTube. Redirector называет кластер
/// провайдера («31.171.198.212 => tktelecom-kuf1 (…)»), а узел этого кластера
/// отвечает по имени `r1.<кластер>.googlevideo.com`.
pub(crate) async fn video_node(client: &reqwest::Client) -> Option<SocketAddr> {
    let url = format!("https://{VIDEO_HOST}/report_mapping?di=no");
    let text = client.get(url).send().await.ok()?.text().await.ok()?;
    let cluster = cluster_name(&text)?;
    let host = format!("r1.{cluster}.googlevideo.com:443");
    tokio::net::lookup_host(host).await.ok()?.next()
}

fn cluster_name(report: &str) -> Option<&str> {
    let name = report.split("=>").nth(1)?.split_whitespace().next()?;
    let valid = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    valid.then_some(name)
}

#[cfg(test)]
mod tests {
    use super::cluster_name;

    /// Ходит в сеть, поэтому запускается вручную: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn connect_time_is_measured() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let ms = runtime.block_on(super::connect_ms(
            "https://discord.com",
            std::time::Duration::from_secs(4),
        ));
        // `None` тоже возможен: через VPN-туннель соединение принимается локально.
        println!("discord.com connect: {ms:?} ms");
        assert_eq!(
            runtime.block_on(super::connect_ms(
                "not a url",
                std::time::Duration::from_secs(1)
            )),
            None
        );
    }

    #[test]
    fn reads_cluster_from_report_mapping() {
        assert_eq!(
            cluster_name("31.171.198.212 => tktelecom-kuf1 (31.171.198.0/24)"),
            Some("tktelecom-kuf1")
        );
        assert_eq!(cluster_name("<html>error</html>"), None);
        assert_eq!(cluster_name("1.2.3.4 => bad.name/x"), None);
    }
}
