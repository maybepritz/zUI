//! Поиск каталогов переводов и потокобезопасный выбор языка.
//! Переводчики правят locales/<код BCP-47>/{locale.json,main.ftl}, а не строки в Rust.
mod text;
pub(crate) mod time;
pub(crate) use text::{Text, text, text_args};

use rust_embed::RustEmbed;
use std::{
    path::Path,
    sync::{OnceLock, RwLock},
};
use z_core::preferences::Language;
use z_i18n::{Catalog, FluentArgs, FluentValue, LocaleInfo, Localizer};

#[derive(RustEmbed)]
#[folder = "locales/"]
struct EmbeddedLocales;

static LOCALIZER: OnceLock<Localizer> = OnceLock::new();
static LANGUAGE: RwLock<String> = RwLock::new(String::new());

fn bundled() -> Localizer {
    let mut localizer = Localizer::new("en");
    for path in EmbeddedLocales::iter().filter(|path| path.ends_with("/locale.json")) {
        let code = path.trim_end_matches("/locale.json");
        let metadata = EmbeddedLocales::get(&path).expect("embedded locale metadata");
        let messages =
            EmbeddedLocales::get(&format!("{code}/main.ftl")).expect("embedded locale messages");
        let catalog = Catalog::parse(
            code,
            &String::from_utf8_lossy(&metadata.data),
            &String::from_utf8_lossy(&messages.data),
        )
        .expect("bundled locales are checked by z-i18n tests");
        localizer.add(catalog);
    }
    localizer
}

/// Загружает установленные переводы один раз при запуске. Ошибочные файлы пользователя попадают в отчёт и пропускаются.
pub(crate) fn initialize(directory: &Path, language: &Language) -> Vec<String> {
    let mut issues = Vec::new();
    if LOCALIZER.get().is_none() {
        let mut localizer = bundled();
        for issue in localizer.load_directory(directory) {
            issues.push(format!("Locale pack: {issue}"));
        }
        let _ = LOCALIZER.set(localizer);
    }
    set_language(language);
    issues
}

pub(crate) fn languages() -> Vec<LocaleInfo> {
    LOCALIZER.get_or_init(bundled).languages()
}

pub(crate) fn set_language(language: &Language) {
    if let Ok(mut current) = LANGUAGE.write() {
        *current = language.code().to_owned();
    }
}

pub(crate) fn resolved_language(language: &Language) -> String {
    LOCALIZER
        .get_or_init(bundled)
        .resolve(language.code())
        .to_owned()
}

pub(crate) fn translators(language: &Language) -> Vec<String> {
    let code = resolved_language(language);
    languages()
        .into_iter()
        .find(|locale| locale.code == code)
        .map(|locale| locale.translators)
        .unwrap_or_default()
}

pub(crate) fn tr(key: &str) -> String {
    tr_args(key, &[])
}

pub(crate) fn tr_args<'a>(key: &str, values: &[(&str, FluentValue<'a>)]) -> String {
    let mut args = FluentArgs::new();
    for (name, value) in values {
        args.set(*name, value.clone());
    }
    let current = LANGUAGE
        .read()
        .map(|value| value.clone())
        .unwrap_or_else(|_| "en".into());
    LOCALIZER
        .get_or_init(bundled)
        .format(&current, key, Some(&args))
}

/// Форматирует первый кадр, не инициализируя основной каталог приложения.
/// Иначе ранний вызов `tr` заставил бы `initialize` пропустить переводы пользователя.
pub(crate) fn startup_tr(key: &str) -> String {
    if LOCALIZER.get().is_some() {
        return tr(key);
    }
    static STARTUP_FALLBACK: OnceLock<Localizer> = OnceLock::new();
    let current = LANGUAGE
        .read()
        .map(|value| value.clone())
        .unwrap_or_else(|_| "en".into());
    STARTUP_FALLBACK
        .get_or_init(bundled)
        .format(&current, key, None)
}
