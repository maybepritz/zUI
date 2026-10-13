//! Application catalog discovery and thread-safe language selection.
//! Translators edit locales/<BCP-47 code>/{locale.json,main.ftl}, never Rust strings.
mod text;
pub(crate) use text::{Text, text, text_args};

use rust_embed::RustEmbed;
use std::{
    fs,
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

/// Load installed packs once at startup. Bad user files are reported and skipped.
pub(crate) fn initialize(directory: &Path, language: &Language) -> Vec<String> {
    let mut issues = Vec::new();
    if LOCALIZER.get().is_none() {
        let mut localizer = bundled();
        match fs::read_dir(directory) {
            Ok(entries) => {
                for entry in entries {
                    let result = (|| -> Result<(), String> {
                        let entry = entry.map_err(|e| e.to_string())?;
                        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                            return Ok(());
                        }
                        let code = entry.file_name().to_string_lossy().into_owned();
                        let metadata = fs::read_to_string(entry.path().join("locale.json"))
                            .map_err(|e| e.to_string())?;
                        let messages = fs::read_to_string(entry.path().join("main.ftl"))
                            .map_err(|e| e.to_string())?;
                        localizer.add(Catalog::parse(&code, &metadata, &messages)?);
                        Ok(())
                    })();
                    if let Err(error) = result {
                        issues.push(format!("Locale pack: {error}"));
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => issues.push(format!("Locale directory: {error}")),
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
