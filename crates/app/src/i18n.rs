use std::sync::atomic::{AtomicBool, Ordering};
use z_core::preferences::Language;
static ENGLISH: AtomicBool = AtomicBool::new(false);

pub fn set_language(language: Language) {
    ENGLISH.store(language == Language::En, Ordering::Relaxed);
}

pub fn tr(ru: &'static str, en: &'static str) -> &'static str {
    if ENGLISH.load(Ordering::Relaxed) {
        en
    } else {
        ru
    }
}
