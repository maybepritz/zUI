//telegram_config.rs
use std::net::{IpAddr, SocketAddr};

pub(crate) fn endpoint(host: &str, port: u64) -> Result<SocketAddr, &'static str> {
    let host: IpAddr = host.parse().map_err(|_| "Некорректный адрес прокси")?;
    if !host.is_loopback() {
        return Err("Используйте локальный адрес 127.0.0.1 или ::1");
    }
    if !(1..=65535).contains(&port) {
        return Err("Порт должен быть числом от 1 до 65535");
    }
    Ok(SocketAddr::new(host, port as u16))
}

pub(crate) fn link(endpoint: SocketAddr, secret: &str) -> Result<String, &'static str> {
    if secret.len() != 32 || !secret.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Secret должен содержать 32 шестнадцатеричных символа");
    }
    // Matches tg_proxy_url in Flowseal's tray application.
    let host = endpoint.ip().to_string().replace(':', "%3A");
    Ok(format!(
        "tg://proxy?server={host}&port={}&secret=dd{secret}",
        endpoint.port()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    const SECRET: &str = "0123456789abcdef0123456789abcdef";
    #[test]
    fn flowseal_link() {
        assert_eq!(
            link(endpoint("127.0.0.1", 1443).unwrap(), SECRET).unwrap(),
            "tg://proxy?server=127.0.0.1&port=1443&secret=dd0123456789abcdef0123456789abcdef"
        );
    }
    #[test]
    fn ipv6_link_is_encoded() {
        assert!(
            link(endpoint("::1", 1444).unwrap(), SECRET)
                .unwrap()
                .contains("server=%3A%3A1&port=1444")
        );
    }
    #[test]
    fn reject_invalid_or_public_endpoints() {
        for port in [0, 65536, u64::MAX] {
            assert!(endpoint("127.0.0.1", port).is_err());
        }
        for host in ["0.0.0.0", "::", "8.8.8.8", "localhost&port=80", "invalid"] {
            assert!(endpoint(host, 1443).is_err());
        }
        assert!(endpoint("127.0.0.1", 65535).is_ok());
    }
    #[test]
    fn reject_bad_secrets() {
        for secret in [
            "",
            "bad&port=80",
            "zz23456789abcdef0123456789abcdef",
            "dd0123456789abcdef0123456789abcdef",
        ] {
            assert!(link(endpoint("127.0.0.1", 1443).unwrap(), secret).is_err());
        }
    }
}
