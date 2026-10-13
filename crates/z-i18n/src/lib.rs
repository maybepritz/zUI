//! Fluent catalogs and validation, independent of GPUI and application services.
mod catalog;
mod files;
mod validation;

pub use catalog::{Catalog, LocaleInfo, Localizer};
pub use files::load_pack;
pub use fluent_bundle::{FluentArgs, FluentValue};
pub use validation::validate;
