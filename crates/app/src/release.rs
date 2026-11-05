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
