//! Определение VPN и прокси, через которые трафик идёт мимо обхода.
//!
//! Одного признака на все VPN нет, поэтому их несколько. Главный — маршрут:
//! через какой адаптер Windows отправит пакеты к адресу сервиса. У настоящей
//! сетевой карты и Wi-Fi Windows ставит флаг «аппаратный интерфейс», у туннелей
//! (sing-box, Clash, WireGuard, OpenVPN, встроенный VPN Windows) его нет.
//! Адаптеры Hyper-V, WSL и VMware тоже виртуальные, но интернет через них
//! не ходит, поэтому в маршрут к сервису они не попадают.
//!
//! Остальные признаки ловят то, что маршрут пропускает: поддельные адреса
//! fake-ip, соединение, принятое на этом компьютере (см. [`crate::net::connect`]),
//! и системный прокси, при котором браузер ходит через прокси без всякого адаптера.
//! Название VPN берётся из списка известных программ и нужно только для текста.
use std::net::IpAddr;

/// Сетевой адаптер, через который Windows отправит пакеты к адресу.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adapter {
    /// Описание адаптера, например `sing-tun Tunnel`.
    pub name: String,
    /// Настоящая сетевая карта или Wi-Fi, а не туннель.
    pub physical: bool,
}

/// Что известно о VPN на момент проверки сервисов.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Vpn {
    /// Виртуальный адаптер, через который идёт хотя бы один сервис.
    pub adapter: Option<String>,
    /// Включённый системный прокси Windows: адрес сервера или PAC-скрипта.
    pub proxy: Option<String>,
    /// Название запущенной VPN-программы, если она известна.
    pub app: Option<String>,
}

impl Vpn {
    /// Как назвать VPN в интерфейсе: программа, иначе адаптер.
    pub fn name(&self) -> Option<&str> {
        self.app.as_deref().or(self.adapter.as_deref())
    }
}

/// Адаптер для пакетов к `ip`. `None`, если Windows не ответила.
pub fn route(ip: IpAddr) -> Option<Adapter> {
    imp::route(ip)
}

/// Адрес из 198.18.0.0/15. Так на DNS отвечают sing-box и Clash в режиме fake-ip:
/// настоящий адрес знает только VPN, а сайт открывается через него.
pub fn is_fake_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            a == 198 && (b == 18 || b == 19)
        }
        IpAddr::V6(_) => false,
    }
}

/// Системный прокси Windows, если он включён: адрес сервера или PAC-скрипта.
/// Проверки zUI его не используют, а браузер ходит через него.
pub fn system_proxy() -> Option<String> {
    imp::system_proxy()
}

/// Известная VPN-программа среди запущенных процессов.
pub fn known_app() -> Option<&'static str> {
    app_name(&imp::process_names())
}

/// Начало имени процесса в нижнем регистре и название для интерфейса.
/// Сначала программы с окном, потом ядра, которые они запускают: у Throne
/// работает и `throne.exe`, и `sing-box.exe`, а назвать стоит Throne.
const APPS: &[(&str, &str)] = &[
    ("throne", "Throne"),
    ("nekoray", "NekoRay"),
    ("nekobox", "NekoBox"),
    ("v2rayn", "v2rayN"),
    ("v2raytun", "v2RayTun"),
    ("clash-verge", "Clash Verge"),
    ("clash verge", "Clash Verge"),
    ("clash-nyanpasu", "Clash Nyanpasu"),
    ("flclash", "FlClash"),
    ("hiddify", "Hiddify"),
    ("karing", "Karing"),
    ("happ", "Happ"),
    ("amneziavpn", "AmneziaVPN"),
    ("amneziawg", "AmneziaWG"),
    ("outline", "Outline"),
    ("wireguard", "WireGuard"),
    ("openvpn", "OpenVPN"),
    ("protonvpn", "Proton VPN"),
    ("windscribe", "Windscribe"),
    ("nordvpn", "NordVPN"),
    ("expressvpn", "ExpressVPN"),
    ("adguardvpn", "AdGuard VPN"),
    ("warp-svc", "Cloudflare WARP"),
    ("psiphon", "Psiphon"),
    ("lantern", "Lantern"),
    ("mihomo", "mihomo"),
    ("clash", "Clash"),
    ("sing-box", "sing-box"),
    ("xray", "Xray"),
    ("v2ray", "V2Ray"),
];

fn app_name(processes: &[String]) -> Option<&'static str> {
    APPS.iter()
        .find(|(prefix, _)| processes.iter().any(|name| name.starts_with(prefix)))
        .map(|(_, name)| *name)
}

#[cfg(windows)]
mod imp {
    use super::Adapter;
    use std::net::IpAddr;
    use windows::Win32::{
        Foundation::{CloseHandle, NO_ERROR},
        NetworkManagement::IpHelper::{GetBestInterfaceEx, GetIfEntry2, MIB_IF_ROW2},
        Networking::WinSock::{
            AF_INET, AF_INET6, IN_ADDR, IN_ADDR_0, IN6_ADDR, IN6_ADDR_0, SOCKADDR, SOCKADDR_IN,
            SOCKADDR_IN6,
        },
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
    };

    /// Флаг `HardwareInterface` в `InterfaceAndOperStatusFlags`.
    const HARDWARE_INTERFACE: u8 = 1;

    pub(super) fn route(ip: IpAddr) -> Option<Adapter> {
        let mut index = 0u32;
        // SAFETY: адрес живёт до конца вызова, его размер соответствует семейству.
        let status = match ip {
            IpAddr::V4(v4) => {
                let address = SOCKADDR_IN {
                    sin_family: AF_INET,
                    sin_addr: IN_ADDR {
                        S_un: IN_ADDR_0 {
                            S_addr: u32::from_ne_bytes(v4.octets()),
                        },
                    },
                    ..Default::default()
                };
                unsafe {
                    GetBestInterfaceEx(std::ptr::from_ref(&address).cast::<SOCKADDR>(), &mut index)
                }
            }
            IpAddr::V6(v6) => {
                let address = SOCKADDR_IN6 {
                    sin6_family: AF_INET6,
                    sin6_addr: IN6_ADDR {
                        u: IN6_ADDR_0 { Byte: v6.octets() },
                    },
                    ..Default::default()
                };
                unsafe {
                    GetBestInterfaceEx(std::ptr::from_ref(&address).cast::<SOCKADDR>(), &mut index)
                }
            }
        };
        if status != NO_ERROR.0 {
            return None;
        }
        let mut row = MIB_IF_ROW2 {
            InterfaceIndex: index,
            ..Default::default()
        };
        // SAFETY: строка заполнена нулями, кроме индекса, как требует GetIfEntry2.
        if unsafe { GetIfEntry2(&mut row) } != NO_ERROR {
            return None;
        }
        Some(Adapter {
            name: wide(&row.Description),
            physical: row.InterfaceAndOperStatusFlags._bitfield & HARDWARE_INTERFACE != 0,
        })
    }

    pub(super) fn system_proxy() -> Option<String> {
        use winreg::{RegKey, enums::HKEY_CURRENT_USER};
        let key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Internet Settings")
            .ok()?;
        // PAC-скрипт работает и без ProxyEnable.
        if let Ok(script) = key.get_value::<String, _>("AutoConfigURL")
            && !script.trim().is_empty()
        {
            return Some(script.trim().to_owned());
        }
        let enabled: u32 = key.get_value("ProxyEnable").ok()?;
        let server: String = key.get_value("ProxyServer").ok()?;
        (enabled != 0 && !server.trim().is_empty()).then(|| server.trim().to_owned())
    }

    pub(super) fn process_names() -> Vec<String> {
        let mut names = Vec::new();
        // SAFETY: снимок закрывается здесь же, запись процесса инициализирована с dwSize.
        unsafe {
            let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
                return names;
            };
            let mut entry = PROCESSENTRY32W {
                dwSize: size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut next = Process32FirstW(snapshot, &mut entry);
            while next.is_ok() {
                names.push(wide(&entry.szExeFile).to_lowercase());
                next = Process32NextW(snapshot, &mut entry);
            }
            let _ = CloseHandle(snapshot);
        }
        names
    }

    fn wide(text: &[u16]) -> String {
        let end = text.iter().position(|&c| c == 0).unwrap_or(text.len());
        String::from_utf16_lossy(&text[..end])
    }
}

#[cfg(not(windows))]
mod imp {
    use super::Adapter;
    use std::net::IpAddr;

    pub(super) fn route(_: IpAddr) -> Option<Adapter> {
        None
    }

    pub(super) fn system_proxy() -> Option<String> {
        None
    }

    pub(super) fn process_names() -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_ip_range_is_198_18_slash_15() {
        assert!(is_fake_ip("198.18.0.5".parse().unwrap()));
        assert!(is_fake_ip("198.19.255.1".parse().unwrap()));
        assert!(!is_fake_ip("198.20.0.1".parse().unwrap()));
        assert!(!is_fake_ip("142.251.150.4".parse().unwrap()));
    }

    #[test]
    fn gui_app_wins_over_its_core() {
        let processes = ["sing-box.exe".to_owned(), "throne.exe".to_owned()];
        assert_eq!(app_name(&processes), Some("Throne"));
        let processes = ["mihomo.exe".to_owned(), "clash-verge.exe".to_owned()];
        assert_eq!(app_name(&processes), Some("Clash Verge"));
        assert_eq!(app_name(&["explorer.exe".to_owned()]), None);
    }

    #[test]
    fn name_prefers_the_app() {
        let vpn = Vpn {
            adapter: Some("sing-tun Tunnel".into()),
            app: Some("Throne".into()),
            ..Vpn::default()
        };
        assert_eq!(vpn.name(), Some("Throne"));
        let vpn = Vpn {
            adapter: Some("WireGuard Tunnel".into()),
            ..Vpn::default()
        };
        assert_eq!(vpn.name(), Some("WireGuard Tunnel"));
    }

    /// Ходит в Windows, поэтому запускается вручную: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn prints_what_this_machine_has() {
        let ip = "8.8.8.8".parse().unwrap();
        println!("route 8.8.8.8: {:?}", route(ip));
        println!("system proxy: {:?}", system_proxy());
        println!("known app: {:?}", known_app());
    }
}
