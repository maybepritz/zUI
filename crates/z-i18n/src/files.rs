//! Filesystem packs are shared by the application and the standalone checker.
use crate::{Catalog, Localizer};
use std::{collections::BTreeSet, fs, path::Path};

pub fn load_pack(path: &Path) -> Result<Catalog, String> {
    let code = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("Invalid locale directory")?;
    let metadata = fs::read_to_string(path.join("locale.json"))
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let messages = fs::read_to_string(path.join("main.ftl"))
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Catalog::parse(code, &metadata, &messages)
}

impl Localizer {
    /// Discover installed packs without an enum or registration file.
    /// Errors skip only the affected pack; a missing directory is normal.
    pub fn load_directory(&mut self, directory: &Path) -> Vec<String> {
        let mut issues = Vec::new();
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return issues,
            Err(error) => return vec![format!("{}: {error}", directory.display())],
        };
        let mut paths = Vec::new();
        for entry in entries {
            match entry.and_then(|entry| Ok((entry.file_type()?.is_dir(), entry.path()))) {
                Ok((true, path)) => paths.push(path),
                Ok((false, _)) => {}
                Err(error) => issues.push(error.to_string()),
            }
        }
        paths.sort();
        let mut seen = BTreeSet::new();
        for path in paths {
            match load_pack(&path) {
                Ok(catalog) if seen.insert(catalog.info.code.clone()) => self.add(catalog),
                Ok(catalog) => issues.push(format!(
                    "{}: duplicate locale {}",
                    path.display(),
                    catalog.info.code
                )),
                Err(error) => issues.push(error),
            }
        }
        issues
    }
}
