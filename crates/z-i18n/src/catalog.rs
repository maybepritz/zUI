use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};
use serde::Deserialize;
use std::collections::BTreeMap;
use unic_langid::LanguageIdentifier;

/// Native language names let people select a locale before understanding the UI.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocaleInfo {
    #[serde(skip)]
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub translators: Vec<String>,
}

pub struct Catalog {
    pub info: LocaleInfo,
    pub(crate) resource: String,
    bundle: FluentBundle<FluentResource>,
}

impl Catalog {
    /// Reject malformed resources and duplicate entries instead of silently losing text.
    pub fn parse(code: &str, metadata: &str, text: &str) -> Result<Self, String> {
        let locale: LanguageIdentifier = code.parse().map_err(|e| format!("{code}: {e}"))?;
        let mut info: LocaleInfo =
            serde_json::from_str(metadata).map_err(|e| format!("{code}: {e}"))?;
        if info.name.trim().is_empty() {
            return Err(format!("{code}: native language name is empty"));
        }
        info.code = locale.to_string();
        let resource = FluentResource::try_new(text.to_owned())
            .map_err(|(_, errors)| format!("{code}: Fluent syntax errors: {errors:?}"))?;
        let mut bundle = FluentBundle::new_concurrent(vec![locale]);
        bundle
            .add_builtins()
            .map_err(|e| format!("{code}: {e:?}"))?;
        bundle
            .add_resource(resource)
            .map_err(|e| format!("{code}: {e:?}"))?;
        Ok(Self {
            info,
            resource: text.to_owned(),
            bundle,
        })
    }

    pub(crate) fn format(&self, key: &str, args: Option<&FluentArgs<'_>>) -> Option<String> {
        let pattern = self.bundle.get_message(key)?.value()?;
        let mut errors = Vec::new();
        let text = self.bundle.format_pattern(pattern, args, &mut errors);
        errors.is_empty().then(|| text.into_owned())
    }
}

/// Multiple catalogs for a language allow a local pack to override bundled messages.
pub struct Localizer {
    catalogs: BTreeMap<String, Vec<Catalog>>,
    fallback: String,
}

impl Localizer {
    pub fn new(fallback: &str) -> Self {
        Self {
            catalogs: BTreeMap::new(),
            fallback: fallback.to_owned(),
        }
    }

    pub fn add(&mut self, catalog: Catalog) {
        self.catalogs
            .entry(catalog.info.code.clone())
            .or_default()
            .push(catalog);
    }

    pub fn languages(&self) -> Vec<LocaleInfo> {
        self.catalogs
            .values()
            .filter_map(|layers| layers.last().map(|c| c.info.clone()))
            .collect()
    }

    pub fn contains(&self, locale: &str, key: &str) -> bool {
        self.catalogs.get(locale).is_some_and(|layers| {
            layers.iter().any(|catalog| {
                catalog
                    .bundle
                    .get_message(key)
                    .is_some_and(|message| message.value().is_some())
            })
        })
    }

    /// Prefer exact BCP-47 locale, then its language subtag, then English.
    pub fn resolve(&self, requested: &str) -> &str {
        if let Ok(id) = requested.parse::<LanguageIdentifier>() {
            let canonical = id.to_string();
            if let Some((code, _)) = self.catalogs.get_key_value(&canonical) {
                return code;
            }
            if let Some((code, _)) = self.catalogs.get_key_value(id.language.as_str()) {
                return code;
            }
        }
        &self.fallback
    }

    /// Missing or invalid messages fall back per key, including partial local packs.
    pub fn format(&self, locale: &str, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        let selected = self.resolve(locale);
        for code in [selected, self.fallback.as_str()] {
            if let Some(layers) = self.catalogs.get(code) {
                for catalog in layers.iter().rev() {
                    if let Some(text) = catalog.format(key, args) {
                        return text;
                    }
                }
            }
        }
        // A visible key is preferable to an empty button or a crash.
        key.to_owned()
    }
}
