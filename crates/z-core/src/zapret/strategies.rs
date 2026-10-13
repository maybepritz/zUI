//! Load strategies and validate their referenced files before starting winws.
use super::Strategy;
#[cfg(test)]
use super::strategy_parser::parse_bat_args;
use super::strategy_parser::{game_ports, parse_with_game_filter, strategy_sort_key};
use anyhow::{Context as _, Result, bail};
use std::{fs, path::Path};

/// Убедиться, что пользовательские списки присутствуют
pub fn ensure_user_lists(lists_dir: &Path) -> Result<()> {
    fs::create_dir_all(lists_dir)?;
    let files = [
        ("ipset-exclude-user.txt", "203.0.113.113/32\r\n"),
        (
            "list-general-user.txt",
            "# Пользовательский список доменов\r\ndomain.example.abc\r\n",
        ),
        (
            "list-exclude-user.txt",
            "# Исключения доменов\r\ndomain.example.abc\r\n",
        ),
    ];
    for (name, default_content) in files {
        let path = lists_dir.join(name);
        if !path.exists() {
            fs::write(&path, default_content)?;
        }
    }
    Ok(())
}

/// Парсинг всех доступных стратегий (.bat файлов) из каталога zapret
pub fn load_strategies(zapret_dir: &Path) -> Result<Vec<Strategy>> {
    let mut strategies = Vec::new();
    let entries = fs::read_dir(zapret_dir)
        .with_context(|| format!("Не удалось прочитать каталог {}", zapret_dir.display()))?;

    let bin_dir = zapret_dir.join("bin");
    let lists_dir = zapret_dir.join("lists");

    let bin_str = format!("{}\\", bin_dir.display());
    let lists_str = format!("{}\\", lists_dir.display());
    let game = game_ports(
        &fs::read_to_string(zapret_dir.join("utils/game_filter.enabled")).unwrap_or_default(),
    );

    for entry in entries.flatten() {
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("bat"))
        {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.eq_ignore_ascii_case("service.bat") {
            continue;
        }

        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(raw_args) =
                parse_with_game_filter(&content, &bin_str, &lists_str, (&game.0, &game.1))
            {
                strategies.push(Strategy {
                    name,
                    file_name,
                    raw_args,
                });
            }
        }
    }

    // Сортировка: general первая, затем ALT по возрастанию, затем FAKE TLS, затем SIMPLE FAKE, затем EXP
    strategies.sort_by(|a, b| strategy_sort_key(&a.name).cmp(&strategy_sort_key(&b.name)));

    Ok(strategies)
}

pub(super) fn validate_args(args: &str) -> Result<()> {
    if args.is_empty() || args.contains('%') {
        bail!("Стратегия пуста или содержит неизвестные переменные BAT");
    }
    // All file arguments in the supported Flowseal strategies are quoted.
    for (index, part) in args.split('"').enumerate() {
        if index % 2 == 1 && !Path::new(part).is_file() {
            bail!("Не найден файл стратегии: {part}");
        }
    }
    if args.matches('"').count() % 2 != 0 {
        bail!("Незакрытая кавычка в стратегии");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_strategies() {
        let zapret_dir = Path::new("../../runtime/zapret");
        if zapret_dir.exists() {
            let strats = load_strategies(zapret_dir).expect("Failed to load strategies");
            assert!(!strats.is_empty(), "Strategies should not be empty");
            assert_eq!(strats[0].name, "general");
            assert!(strats[0].raw_args.contains("--wf-tcp="));
        }
    }

    #[test]
    fn parser_ignores_comments_and_preserves_unicode_and_spaces() {
        let content = "rem winws.exe --wrong\nstart \"Тест\" \"%bin%winws.exe\" --wf-tcp=443 ^\n --hostlist=\"%lists%list.txt\"\necho ignored";
        assert_eq!(
            parse_bat_args(content, "C:\\Путь с пробелом\\", "C:\\списки\\").unwrap(),
            "--wf-tcp=443 --hostlist=\"C:\\списки\\list.txt\""
        );
    }

    #[test]
    fn parser_handles_unquoted_executable_and_empty_command() {
        assert_eq!(
            parse_bat_args("winws.exe --filter-tcp=%GameFilterTCP%", "", "").unwrap(),
            "--filter-tcp=12"
        );
        assert!(parse_bat_args("winws.exe\"", "", "").is_none());
        assert!(validate_args("--hostlist=%UNKNOWN%").is_err());
    }
}
