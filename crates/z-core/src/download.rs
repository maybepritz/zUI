use anyhow::{Result, bail};

pub fn verify(expected_size: u64, actual_size: u64, expected: &str, actual: &str) -> Result<()> {
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("Отсутствует корректный SHA-256 официального релиза");
    }
    if actual_size != expected_size || !actual.eq_ignore_ascii_case(expected) {
        bail!("Проверка размера или SHA-256 не пройдена");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_tampered_truncated_or_unsigned_downloads() {
        let digest = "a".repeat(64);
        assert!(verify(10, 10, &digest, &digest.to_uppercase()).is_ok());
        assert!(verify(10, 9, &digest, &digest).is_err());
        assert!(verify(10, 10, &digest, &"b".repeat(64)).is_err());
        assert!(verify(10, 10, "", "").is_err());
    }
}
