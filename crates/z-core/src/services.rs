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
    if id == TELEGRAM {
        return "Telegram";
    }
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
    /// Сервис идёт через VPN или прокси, а не через обход: его сбой — не повод
    /// менять стратегию, а задержка показывает VPN.
    pub via_vpn: bool,
}

#[derive(Clone, Debug)]
pub struct CheckBatch {
    pub checks: Vec<Check>,
    pub internet_available: bool,
    pub checked_at: u64,
    pub vpn: crate::vpn::Vpn,
    /// Подключён ли клиент Telegram к прокси; `None`, если прокси не проверялся.
    pub telegram_client: Option<bool>,
}

/// Id Telegram среди проверок. Это не сервис из [`SERVICES`]: его проверяют,
/// пока работает прокси, и zapret его не чинит.
pub const TELEGRAM: &str = "telegram";

/// Куда ходит Telegram-прокси: проверять нужно тот же сервер.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramTarget {
    /// Имя WebSocket-сервера Telegram, например `kws2.web.telegram.org`.
    pub host: String,
    /// Адрес из настроек прокси (`dc_ip`); `None` — прокси узнаёт его через DNS.
    pub address: Option<std::net::SocketAddr>,
    /// Локальный порт прокси, к которому подключается клиент Telegram.
    pub port: u16,
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

/// Итог проверки одного адреса до того, как он станет [`Check`].
struct Probe {
    availability: Availability,
    latency_ms: u64,
    via_vpn: bool,
    /// Виртуальный адаптер на пути к серверу.
    adapter: Option<String>,
}

/// Открывает адрес, замеряет пинг до `address` и смотрит, не идёт ли он через VPN.
async fn probe_address(
    client: &reqwest::Client,
    url: &str,
    address: Option<std::net::SocketAddr>,
) -> Probe {
    let (availability, response_ms) = probe_twice(client, url).await;
    let route = address.and_then(|a| crate::vpn::route(a.ip()));
    let adapter = route.filter(|r| !r.physical).map(|r| r.name);
    let fake_ip = address.is_some_and(|a| crate::vpn::is_fake_ip(a.ip()));
    // Задержка — время соединения с сервером, а не ответа сайта: так она похожа
    // на пинг в звонке или в браузере. Если соединение принял VPN-туннель на этом
    // компьютере, пинг неизвестен и остаётся время ответа сайта.
    let connect = match address {
        Some(address) if availability.connected() => crate::net::connect(address, CONNECT).await,
        _ => crate::net::Connect::Failed,
    };
    let latency_ms = match connect {
        crate::net::Connect::Remote(ms) => ms,
        _ => response_ms,
    };
    Probe {
        availability,
        latency_ms,
        via_vpn: adapter.is_some() || fake_ip || connect == crate::net::Connect::Local,
        adapter,
    }
}

/// Проверяет сервисы `ids` и, если прокси работает, путь Telegram-прокси.
pub fn check(ids: &[String], telegram: Option<TelegramTarget>) -> anyhow::Result<CheckBatch> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let client = crate::net::client(CONNECT, TOTAL)?;

        let mut jobs = tokio::task::JoinSet::new();
        for service in selected(ids) {
            let client = client.clone();
            jobs.spawn(async move {
                let address = crate::net::resolve(service.url).await;
                let probe = probe_address(&client, service.url, address).await;
                (service.id.to_owned(), probe)
            });
        }
        // Прокси ходит к Telegram через WebSocket веб-версии: тот же сервер, то же имя.
        // Ответ на обычный запрос к /apiws значит, что DPI пропускает рукопожатие.
        let telegram_port = telegram.as_ref().map(|t| t.port);
        if let Some(target) = telegram {
            let url = format!("https://{}/apiws", target.host);
            let address = match target.address {
                Some(address) => Some(address),
                None => crate::net::resolve(&url).await,
            };
            let client = match address {
                Some(address) => crate::net::pinned_client(CONNECT, TOTAL, &target.host, address)?,
                None => client.clone(),
            };
            jobs.spawn(async move {
                let probe = probe_address(&client, &url, address).await;
                (TELEGRAM.to_owned(), probe)
            });
        }
        let (control, _) = crate::net::probe(&client, CONTROL_URL).await;

        let mut checks = Vec::new();
        let mut adapter = None;
        while let Some(result) = jobs.join_next().await {
            let (id, probe) = result?;
            adapter = adapter.or(probe.adapter);
            checks.push(Check {
                id,
                availability: probe.availability,
                latency_ms: probe.latency_ms,
                via_vpn: probe.via_vpn,
            });
        }
        checks.sort_by(|a, b| a.id.cmp(&b.id));

        let any_vpn = checks.iter().any(|c| c.via_vpn);
        let proxy = crate::vpn::system_proxy();
        let vpn = crate::vpn::Vpn {
            app: (any_vpn || proxy.is_some())
                .then(crate::vpn::known_app)
                .flatten()
                .map(str::to_owned),
            adapter,
            proxy,
        };
        Ok(CheckBatch {
            checks,
            internet_available: control.connected(),
            checked_at: crate::clock::now(),
            vpn,
            telegram_client: telegram_port.map(crate::net::has_local_clients),
        })
    })
}
