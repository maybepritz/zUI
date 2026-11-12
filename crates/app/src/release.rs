//! Название выпуска хранится отдельно от числовой версии для Cargo и обновлений.
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn codename() -> &'static str {
    include_str!("../release-name.txt").trim()
}

pub(crate) fn label() -> String {
    format!("{} {}", VERSION, codename())
}

pub(crate) fn title() -> String {
    format!("zUI {}", label())
}

/// Экран изменений описывает именованный выпуск, а исправления внутри него
/// (`0.3.0` → `0.3.1`) имя и заметки не меняют. Поэтому сравниваем без последнего числа.
pub(crate) fn same_release(a: &str, b: &str) -> bool {
    fn major_minor(version: &str) -> Option<(&str, &str)> {
        let mut parts = version.split('.');
        Some((parts.next()?, parts.next()?))
    }
    major_minor(a).is_some_and(|x| major_minor(b) == Some(x))
}

#[cfg(test)]
mod tests {
    use super::same_release;

    #[test]
    fn patch_versions_belong_to_one_release() {
        assert!(same_release("0.3.0", "0.3.1"));
        assert!(!same_release("0.2.0", "0.3.1"));
        assert!(!same_release("1.3.0", "0.3.0"));
        assert!(!same_release("", "0.3.1"));
    }
}
