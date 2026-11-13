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
    pinned_client(connect, total, VIDEO_HOST, node)
}

/// Клиент, который ходит на `host` по адресу `address`, а не по DNS:
/// имя нужно для сертификата, а соединение идёт туда же, куда и у программы.
pub(crate) fn pinned_client(
    connect: Duration,
    total: Duration,
    host: &str,
    address: SocketAddr,
) -> reqwest::Result<reqwest::Client> {
    builder(connect, total).resolve(host, address).build()
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

/// Адрес сервера для `url`, как его вернёт DNS Windows.
pub(crate) async fn resolve(url: &str) -> Option<SocketAddr> {
    let url = reqwest::Url::parse(url).ok()?;
    let host = url.host_str()?;
    let port = url.port_or_known_default()?;
    tokio::net::lookup_host((host, port)).await.ok()?.next()
}

/// Чем закончилась установка TCP-соединения.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Connect {
    /// Соединился сервер, время в миллисекундах — почти чистый сетевой пинг.
    Remote(u64),
    /// Соединение принято на этом компьютере за 0–1 мс: так делает VPN-туннель
    /// (sing-box, Clash, WireGuard), а до сервера на самом деле далеко.
    Local,
    Failed,
}

/// Быстрее этого соединение с удалённым сервером не устанавливается.
const LOCAL_CONNECT_MS: u64 = 2;

/// Устанавливает TCP-соединение и замеряет время. В отличие от [`probe`], сюда
/// не входят TLS, обход и работа веб-сервера, поэтому цифра сравнима с пингом
/// в звонке или в браузере. Адрес узнаётся заранее: время DNS к пингу не относится.
pub(crate) async fn connect(address: SocketAddr, timeout: Duration) -> Connect {
    let started = Instant::now();
    match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(address)).await {
        Ok(Ok(_)) => match started.elapsed().as_millis() as u64 {
            ms if ms < LOCAL_CONNECT_MS => Connect::Local,
            ms => Connect::Remote(ms),
        },
        _ => Connect::Failed,
    }
}

/// Есть ли установленные соединения к локальному порту `port`, то есть подключён ли
/// к прокси хоть один клиент. Сам прокси об этом не сообщает, а Windows знает.
pub(crate) fn has_local_clients(port: u16) -> bool {
    imp::has_local_clients(port)
}

#[cfg(windows)]
mod imp {
    use windows::Win32::{
        Foundation::NO_ERROR,
        NetworkManagement::IpHelper::{
            GetExtendedTcpTable, MIB_TCP_STATE_ESTAB, MIB_TCPROW_OWNER_PID,
            TCP_TABLE_OWNER_PID_CONNECTIONS,
        },
        Networking::WinSock::AF_INET,
    };

    pub(super) fn has_local_clients(port: u16) -> bool {
        // Порт в таблице хранится в сетевом порядке байт в младших 16 битах.
        let wanted = u32::from(port.to_be());
        let mut size = 0u32;
        // SAFETY: первый вызов только узнаёт размер; второй пишет в буфер этого размера.
        unsafe {
            GetExtendedTcpTable(
                None,
                &mut size,
                false,
                u32::from(AF_INET.0),
                TCP_TABLE_OWNER_PID_CONNECTIONS,
                0,
            );
            // Запас на соединения, появившиеся между вызовами.
            size += 16 * size_of::<MIB_TCPROW_OWNER_PID>() as u32;
            let mut buffer = vec![0u32; (size as usize).div_ceil(4)];
            if GetExtendedTcpTable(
                Some(buffer.as_mut_ptr().cast()),
                &mut size,
                false,
                u32::from(AF_INET.0),
                TCP_TABLE_OWNER_PID_CONNECTIONS,
                0,
            ) != NO_ERROR.0
            {
                return false;
            }
            let count = buffer[0] as usize;
            let rows = std::slice::from_raw_parts(
                buffer.as_ptr().add(1).cast::<MIB_TCPROW_OWNER_PID>(),
                count,
            );
            // Клиент — сторона, у которой удалённый порт равен порту прокси на loopback.
            rows.iter().any(|row| {
                row.dwState == MIB_TCP_STATE_ESTAB.0 as u32
                    && row.dwRemotePort == wanted
                    && u32::from_be(row.dwRemoteAddr) >> 24 == 127
            })
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub(super) fn has_local_clients(_: u16) -> bool {
        false
    }
}

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
        let address = runtime.block_on(super::resolve("https://discord.com"));
        println!("discord.com: {address:?}");
        let address = address.unwrap();
        // `Local` тоже возможен: через VPN-туннель соединение принимается на компьютере.
        let result = runtime.block_on(super::connect(address, std::time::Duration::from_secs(4)));
        println!("connect: {result:?}");
        assert_ne!(result, super::Connect::Failed);
        assert_eq!(runtime.block_on(super::resolve("not a url")), None);
    }

    /// Смотрит таблицу соединений Windows, поэтому запускается вручную.
    #[test]
    #[ignore]
    fn reads_the_tcp_table() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!super::has_local_clients(port));
        let _client = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        let _server = listener.accept().unwrap();
        assert!(super::has_local_clients(port));
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
