//! Standalone checker: translators do not need to compile GPUI to validate a pack.
use std::{fs, path::PathBuf, process::ExitCode};
use z_i18n::{load_pack, validate};

fn run() -> Result<(), String> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../app/locales"));
    let reference = load_pack(&root.join("en"))?;
    let mut failed = false;
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if !path.is_dir() {
            continue;
        }
        match load_pack(&path)
            .and_then(|catalog| validate(&reference, &catalog).map_err(|e| e.join("\n")))
        {
            Ok(()) => println!("{}: OK", path.display()),
            Err(error) => {
                eprintln!("{}:\n{error}", path.display());
                failed = true;
            }
        }
    }
    if failed {
        Err("Locale validation failed".into())
    } else {
        Ok(())
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
