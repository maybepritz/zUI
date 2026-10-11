pub(crate) fn strategy_sort_key(name: &str) -> (u8, usize, String) {
    if name == "general" {
        return (0, 0, name.to_string());
    }
    if name.starts_with("general (ALT") {
        // extract number if any
        let num_part = name
            .trim_start_matches("general (ALT")
            .trim_end_matches(')');
        let num: usize = num_part.parse().unwrap_or(1);
        return (1, num, name.to_string());
    }
    if name.starts_with("general (FAKE TLS") {
        return (2, 0, name.to_string());
    }
    if name.starts_with("general (SIMPLE FAKE") {
        return (3, 0, name.to_string());
    }
    if name.starts_with("general (EXP") {
        return (4, 0, name.to_string());
    }
    (5, 0, name.to_string())
}

/// Извлечение аргументов команды winws.exe из текста .bat файла
#[cfg(test)]
pub(crate) fn parse_bat_args(content: &str, bin_str: &str, lists_str: &str) -> Option<String> {
    parse_with_game_filter(content, bin_str, lists_str, ("12", "12"))
}

pub(crate) fn parse_with_game_filter(content: &str, bin_str: &str, lists_str: &str, ports: (&str, &str)) -> Option<String> {
    // ASCII case conversion preserves UTF-8 byte offsets, unlike to_lowercase.
    // Ignore mentions in comments and stop exactly at the end of the command.
    let command = content.lines().find(|line| {
        let lower = line.trim().to_ascii_lowercase();
        !lower.starts_with("rem ") && !lower.starts_with("::")
            && (lower.contains("winws.exe\"") || lower.contains("winws.exe "))
    })?;
    let offset = command.to_ascii_lowercase().find("winws.exe")? + 9;
    let start = content.find(command)? + offset;
    let remainder = content[start..].strip_prefix('"').unwrap_or(&content[start..]);
    let mut parts = Vec::new();

    for line in remainder.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(without_caret) = trimmed.strip_suffix('^') {
            parts.push(without_caret.trim());
        } else {
            parts.push(trimmed);
            break; // Команда завершилась строкой без символа продолжения '^'
        }
    }

    if parts.is_empty() {
        return None;
    }

    let mut full = parts.join(" ");

    // Замена переменных путей и фильтров
    for (key, value) in [("%bin%", bin_str), ("%lists%", lists_str),
        ("%gamefiltertcp%", ports.0), ("%gamefilterudp%", ports.1), ("%gamefilter%", ports.0)] {
        let indices: Vec<usize> = full.to_ascii_lowercase().match_indices(key).map(|(index, _)| index).collect();
        for index in indices.into_iter().rev() {
            full.replace_range(index..index + key.len(), value);
        }
    }

    // Удаление возможных остаточных одиночных символов '^'
    full = full.replace(" ^ ", " ");

    let full = full.trim();
    if full.is_empty() { None } else { Some(full.to_string()) }
}

pub(crate) fn game_ports(content: &str) -> (String, String) {
    let mut mode = "disabled".to_string();
    let mut tcp = "1024-65535".to_string();
    let mut udp = tcp.clone();
    for line in content.lines() {
        let (key, value) = line.trim().split_once('=').unwrap_or((line.trim(), ""));
        match key.to_ascii_lowercase().as_str() {
            "mode" => mode = value.trim().to_ascii_lowercase(),
            "all" => mode = "all".into(),
            "tcp" if value.is_empty() => mode = "tcp".into(),
            "udp" if value.is_empty() => mode = "udp".into(),
            "tcp" if valid_ports(value) => tcp = value.trim().into(),
            "udp" if valid_ports(value) => udp = value.trim().into(),
            _ => {}
        }
    }
    (if mode == "all" || mode == "tcp" { tcp } else { "12".into() },
     if mode == "all" || mode == "udp" { udp } else { "12".into() })
}

fn valid_ports(value: &str) -> bool {
    !value.trim().is_empty() && value.trim().split(',').all(|range| {
        let (start, end) = range.split_once('-').unwrap_or((range, range));
        matches!((start.parse::<u16>(), end.parse::<u16>()), (Ok(a), Ok(b)) if a > 0 && a <= b)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuation_comments_unicode_and_case() {
        let script = "rem winws.exe --wrong\nstart \"İ Тест\" \"%bin%WINWS.EXE\" --wf-tcp=443 ^\n --hostlist=\"%lists%list.txt\"\necho ignored";
        assert_eq!(parse_bat_args(script, "C:\\Путь с пробелом\\", "C:\\списки\\").unwrap(), "--wf-tcp=443 --hostlist=\"C:\\списки\\list.txt\"");
    }
    #[test]
    fn unquoted_and_empty() {
        assert_eq!(parse_bat_args("winws.exe --filter-tcp=%GameFilterTCP%", "", "").unwrap(), "--filter-tcp=12");
        assert!(parse_bat_args("winws.exe\"", "", "").is_none());
        assert!(parse_bat_args("rem winws.exe --wrong", "", "").is_none());
    }
    #[test]
    fn game_modes_and_invalid_ports() {
        assert_eq!(game_ports(""), ("12".into(), "12".into()));
        assert_eq!(game_ports("mode=udp\nudp=2000-3000"), ("12".into(), "2000-3000".into()));
        assert_eq!(game_ports("mode=all\ntcp=443\nudp=99999"), ("443".into(), "1024-65535".into()));
        assert!(!valid_ports("0")); assert!(!valid_ports("500-100"));
        assert_eq!(parse_with_game_filter("winws.exe --wf-tcp=%gamefiltertcp% --wf-udp=%gamefilterudp%", "", "", ("443", "400-500")).unwrap(), "--wf-tcp=443 --wf-udp=400-500");
    }
    #[test]
    fn natural_strategy_order() {
        let mut names = ["general (ALT10)", "general (ALT2)", "general", "general (ALT)"];
        names.sort_by_key(|name| strategy_sort_key(name));
        assert_eq!(names, ["general", "general (ALT)", "general (ALT2)", "general (ALT10)"]);
    }

    #[test]
    fn bundled_strategies_reference_existing_files() {
        let root = ["runtime/zapret", "../../runtime/zapret"].into_iter()
            .map(std::path::PathBuf::from).find(|p| p.join("bin/winws.exe").is_file());
        let Some(root) = root else { return; };
        let root = std::path::absolute(root).unwrap();
        let bin = format!("{}\\", root.join("bin").display());
        let lists = format!("{}\\", root.join("lists").display());
        let mut count = 0;
        for entry in std::fs::read_dir(&root).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "bat") || path.file_name().unwrap() == "service.bat" { continue; }
            let script = std::fs::read_to_string(&path).unwrap();
            let args = parse_bat_args(&script, &bin, &lists).expect("strategy command missing");
            assert!(!args.contains('%'), "unresolved variable in {}", path.display());
            assert!(args.contains("--wf-tcp="));
            for (i, part) in args.split('"').enumerate() {
                if i % 2 == 1 { assert!(std::path::Path::new(part).is_file(), "missing file: {part}"); }
            }
            count += 1;
        }
        assert!(count > 0);
        println!("Checked {count} bundled strategies");
    }
}

