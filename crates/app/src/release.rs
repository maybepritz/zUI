//! Display names are separate from the numeric version used by Cargo and updates.
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Keep profile data and implementation, but omit the unfinished release feature from UI.
pub(crate) const PROFILES_VISIBLE: bool = false;

pub(crate) fn codename() -> &'static str {
    include_str!("../release-name.txt").trim()
}

pub(crate) fn label() -> String {
    format!("{} {}", VERSION, codename())
}

pub(crate) fn title() -> String {
    format!("zUI {}", label())
}
