//! Deferred status messages update when the selected language changes.
use super::{tr, tr_args};
use gpui_kit::SharedString;
use z_i18n::FluentValue;

#[derive(Clone, Default)]
pub(crate) struct Text {
    key: Option<&'static str>,
    values: Vec<(&'static str, FluentValue<'static>)>,
    literal: String,
    suffix: String,
}

pub(crate) fn text(key: &'static str) -> Text {
    Text {
        key: Some(key),
        ..Default::default()
    }
}

pub(crate) fn text_args(
    key: &'static str,
    values: &[(&'static str, FluentValue<'static>)],
) -> Text {
    Text {
        key: Some(key),
        values: values.to_vec(),
        ..Default::default()
    }
}

impl Text {
    pub(crate) fn is_empty(&self) -> bool {
        self.key.is_none() && self.literal.is_empty()
    }
    pub(crate) fn append_detail(&mut self, detail: &str) {
        self.suffix.push_str(detail);
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.key {
            Some(key) if self.values.is_empty() => f.write_str(&tr(key))?,
            Some(key) => f.write_str(&tr_args(key, &self.values))?,
            None => f.write_str(&self.literal)?,
        }
        f.write_str(&self.suffix)
    }
}

impl From<String> for Text {
    fn from(literal: String) -> Self {
        Self {
            literal,
            ..Default::default()
        }
    }
}
impl From<&str> for Text {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}
impl From<Text> for SharedString {
    fn from(value: Text) -> Self {
        value.to_string().into()
    }
}
