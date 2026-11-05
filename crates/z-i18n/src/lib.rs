//! Каталоги Fluent и их проверка, без зависимости от GPUI и сервисов приложения.
mod catalog;
mod files;
mod validation;

pub use catalog::{Catalog, LocaleInfo, Localizer};
pub use files::load_pack;
pub use fluent_bundle::{FluentArgs, FluentValue};
pub use validation::validate;
