use std::{fs, path::Path, sync::Arc, thread};
use z_i18n::{Catalog, FluentArgs, Localizer, validate};

fn catalog(code: &str, messages: &str) -> Catalog {
    Catalog::parse(code, r#"{"name":"Test language"}"#, messages).unwrap()
}

fn bundled(code: &str) -> Catalog {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../app/locales")
        .join(code);
    Catalog::parse(
        code,
        &fs::read_to_string(root.join("locale.json")).unwrap(),
        &fs::read_to_string(root.join("main.ftl")).unwrap(),
    )
    .unwrap()
}

#[test]
fn builtin_catalogs_have_matching_keys_and_variables() {
    let english = bundled("en");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/locales");
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let code = entry.file_name().to_string_lossy().into_owned();
            validate(&english, &bundled(&code)).unwrap();
        }
    }
}

#[test]
fn partial_local_overrides_fall_back_to_bundled_language_then_english() {
    let mut localizer = Localizer::new("en");
    localizer.add(catalog("en", "hello = Hello\nonly-english = Fallback\n"));
    localizer.add(catalog("ru", "hello = Привет\noriginal = Исходный\n"));
    localizer.add(catalog("ru", "hello = Свой перевод\n"));
    assert_eq!(localizer.format("ru-RU", "hello", None), "Свой перевод");
    assert_eq!(localizer.format("ru", "original", None), "Исходный");
    assert_eq!(localizer.format("ru", "only-english", None), "Fallback");
    assert_eq!(localizer.format("xx", "hello", None), "Hello");
    assert_eq!(localizer.format("ru", "missing", None), "missing");
    assert_eq!(localizer.languages().len(), 2);
}

#[test]
fn invalid_message_resolution_falls_back_instead_of_displaying_broken_variables() {
    let mut localizer = Localizer::new("en");
    localizer.add(catalog("en", "hello = Hello\n"));
    localizer.add(catalog("ru", "hello = Привет, { $missing }\n"));
    assert_eq!(localizer.format("ru", "hello", None), "Hello");
}

#[test]
fn syntax_duplicate_entries_and_missing_native_names_are_rejected() {
    assert!(Catalog::parse("en", r#"{"name":"English"}"#, "broken = {\n").is_err());
    assert!(Catalog::parse("en", r#"{"name":"English"}"#, "one = First\none = Second\n").is_err());
    assert!(Catalog::parse("en", r#"{"name":" "}"#, "one = First\n").is_err());
    assert!(Catalog::parse("../en", r#"{"name":"English"}"#, "one = First\n").is_err());
}

#[test]
fn checker_rejects_missing_keys_changed_variables_and_references_in_other_branches() {
    let reference = catalog(
        "en",
        "greeting = Hello { $name }\nitems = { $count } items\n",
    );
    assert!(
        validate(
            &reference,
            &catalog("ru", "greeting = Привет { $person }\n")
        )
        .is_err()
    );
    assert!(validate(&reference, &catalog("ru", "greeting = Привет { $name }\nitems = { $count ->\n [one] { unknown-message }\n *[other] { $count }\n}\n")).is_err());
    assert!(
        validate(
            &catalog("en", "a = A\nb = B\n"),
            &catalog("ru", "a = { b }\nb = { a }\n")
        )
        .is_err()
    );
}

#[test]
fn references_preserve_the_effective_argument_contract() {
    let reference = catalog(
        "en",
        "base = Hello { $name }\nwelcome = Welcome { $name }\n",
    );
    let translated = catalog("ru", "base = Привет { $name }\nwelcome = { base }\n");
    validate(&reference, &translated).unwrap();
}

#[test]
fn number_builtin_can_select_ordinal_plural_forms() {
    let english = catalog(
        "en",
        "rank = { NUMBER($n, type: \"ordinal\") ->\n    [one] first\n   *[other] other\n}\n",
    );
    validate(&english, &english).unwrap();
    let mut localizer = Localizer::new("en");
    localizer.add(english);
    let mut args = FluentArgs::new();
    args.set("n", 1);
    assert_eq!(localizer.format("en", "rank", Some(&args)), "first");
    args.set("n", 2);
    assert_eq!(localizer.format("en", "rank", Some(&args)), "other");
}

#[test]
fn russian_and_english_plural_forms_use_numeric_arguments() {
    let mut localizer = Localizer::new("en");
    localizer.add(bundled("en"));
    localizer.add(bundled("ru"));
    for (count, suffix) in [(1, "запись"), (2, "записи"), (5, "записей"), (21, "запись")]
    {
        let mut args = FluentArgs::new();
        args.set("count", count);
        let result = localizer.format("ru", "logs-count", Some(&args));
        assert!(result.ends_with(suffix), "{count}: {result}");
    }
    let mut args = FluentArgs::new();
    args.set("count", 1);
    assert!(
        localizer
            .format("en", "logs-count", Some(&args))
            .ends_with("entry")
    );
    args.set("count", 2);
    assert!(
        localizer
            .format("en", "logs-count", Some(&args))
            .ends_with("entries")
    );
}

#[test]
fn formatting_is_safe_across_worker_threads() {
    let mut localizer = Localizer::new("en");
    localizer.add(catalog("en", "hello = Hello { $name }\n"));
    let localizer = Arc::new(localizer);
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let localizer = localizer.clone();
            thread::spawn(move || {
                let mut args = FluentArgs::new();
                args.set("name", "zUI");
                assert!(localizer.format("en", "hello", Some(&args)).contains("zUI"));
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
}

#[test]
fn installed_language_is_discovered_and_broken_neighbor_does_not_disable_it() {
    let temp = std::env::temp_dir();
    let root = temp.join(format!(
        "zui-locales-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("de")).unwrap();
    fs::create_dir_all(root.join("fr")).unwrap();
    fs::write(
        root.join("de/locale.json"),
        r#"{"name":"Deutsch","translators":["Contributor"]}"#,
    )
    .unwrap();
    fs::write(root.join("de/main.ftl"), "hello = Hallo\n").unwrap();
    fs::write(root.join("fr/locale.json"), "invalid JSON").unwrap();
    fs::write(root.join("fr/main.ftl"), "hello = Bonjour\n").unwrap();
    let mut localizer = Localizer::new("en");
    localizer.add(catalog("en", "hello = Hello\n"));
    assert_eq!(localizer.load_directory(&root).len(), 1);
    assert_eq!(localizer.format("de-DE", "hello", None), "Hallo");
    assert_eq!(
        localizer
            .languages()
            .iter()
            .find(|info| info.code == "de")
            .unwrap()
            .translators,
        ["Contributor"]
    );
    assert_eq!(localizer.format("fr", "hello", None), "Hello");
    assert!(localizer.load_directory(&root.join("absent")).is_empty());
    let root = root.canonicalize().unwrap();
    assert!(root.starts_with(temp.canonicalize().unwrap()));
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("zui-locales-")
    );
    fs::remove_dir_all(root).unwrap();
}

fn check_keys(path: &Path, localizer: &Localizer) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_keys(&path, localizer);
            continue;
        }
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        for function in ["tr(", "tr_args(", "text(", "text_args("] {
            for (index, _) in source.match_indices(function) {
                let rest = source[index + function.len()..].trim_start();
                let Some(rest) = rest.strip_prefix('"') else {
                    continue;
                };
                let key = rest.split('"').next().unwrap();
                // Check existence independently of each caller's argument values.
                assert!(
                    localizer.contains("en", key),
                    "{}: unknown key {key}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn literal_message_ids_in_application_sources_exist() {
    let mut localizer = Localizer::new("en");
    localizer.add(bundled("en"));
    check_keys(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src"),
        &localizer,
    );
}
