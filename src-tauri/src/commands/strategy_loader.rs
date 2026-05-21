use super::Strategy;
use crate::commands::apex;
use crate::paths::zapret_dir;
use std::path::Path;

pub fn load_strategies() -> Result<Vec<Strategy>, String> {
    let root = zapret_dir();
    if root.join("bin").join("winws.exe").is_file() {
        let _ = apex::ensure_apex_assets();
        return scan_bat_strategies(&root);
    }
    load_strategies_json_fallback()
}

fn scan_bat_strategies(root: &Path) -> Result<Vec<Strategy>, String> {
    let mut strategies = Vec::new();

    let entries = std::fs::read_dir(root)
        .map_err(|e| format!("Не удалось прочитать папку zapret: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("bat") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if name.eq_ignore_ascii_case("service.bat") {
            continue;
        }
        if !name.starts_with("general") {
            continue;
        }

        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let args = parse_winws_args(&content, root).unwrap_or_default();
        let display_name = bat_display_name(&name);
        let id = bat_to_id(&name);

        let tags = infer_tags(&name, &content);
        let description = if tags.iter().any(|t| t == "apex") {
            "Пресет для Apex Legends (issue #6503): list-apex.txt, порты UDP 10000–10100, ipset-exclude".to_string()
        } else {
            format!("Запуск: {}", name)
        };

        strategies.push(Strategy {
            id,
            name: display_name.clone(),
            description,
            tags,
            source_bat: name,
            args,
        });
    }

    strategies.sort_by(|a, b| bat_sort_key(&a.source_bat).cmp(&bat_sort_key(&b.source_bat)));

    if strategies.is_empty() {
        return load_strategies_json_fallback();
    }

    Ok(strategies)
}

pub fn is_base_general_strategy(id: &str, source_bat: &str) -> bool {
    id == "general" || source_bat.eq_ignore_ascii_case("general.bat")
}

/// Порядок для автоподбора: популярные ALT9/11 раньше, GENERAL в конце.
pub fn autodetect_priority(id: &str, source_bat: &str) -> (u8, u8, u32, String) {
    let upper = source_bat.to_uppercase();
    let tier = if is_base_general_strategy(id, source_bat) {
        3u8
    } else if upper.contains("ALT9") || upper.contains("ALT11") {
        0u8
    } else if upper.contains("ALT10") || upper.contains("ALT3") {
        1u8
    } else if upper.contains("ALT") {
        2u8
    } else {
        2u8
    };
    let (a, b, c) = bat_sort_key(source_bat);
    (tier, a, b, c)
}

fn bat_sort_key(filename: &str) -> (u8, u32, String) {
    let stem = filename.trim_end_matches(".bat");
    if stem == "general" {
        return (0, 0, String::new());
    }
    if stem.eq_ignore_ascii_case("general (APEX)") {
        return (0, 1, "APEX".into());
    }
    if let Some(rest) = stem.strip_prefix("general (") {
        if let Some(inner) = rest.strip_suffix(')') {
            let upper = inner.to_uppercase();
            if upper == "FAKE TLS AUTO" {
                return (1, 0, upper);
            }
            if upper.starts_with("FAKE TLS AUTO ALT") {
                let n = upper
                    .trim_start_matches("FAKE TLS AUTO ALT")
                    .trim()
                    .parse()
                    .unwrap_or(0);
                return (2, n, upper);
            }
            if upper.starts_with("SIMPLE FAKE") {
                let n = if upper == "SIMPLE FAKE" {
                    0
                } else {
                    upper
                        .trim_start_matches("SIMPLE FAKE ALT")
                        .trim()
                        .parse()
                        .unwrap_or(99)
                };
                return (3, n, upper);
            }
            if upper.starts_with("ALT") {
                let n = upper
                    .trim_start_matches("ALT")
                    .trim()
                    .parse()
                    .unwrap_or(999);
                return (4, n, upper);
            }
            return (5, 0, upper);
        }
    }
    (9, 0, stem.to_string())
}

fn load_strategies_json_fallback() -> Result<Vec<Strategy>, String> {
    use crate::paths::data_file_or_err;
    let path = data_file_or_err("strategies.json")?;
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Не удалось прочитать strategies.json: {e}"))?;
    let mut list: Vec<Strategy> =
        serde_json::from_str(&content).map_err(|e| format!("Ошибка парсинга strategies.json: {e}"))?;
    for s in &mut list {
        s.tags.retain(|t| t != "google");
    }
    Ok(list)
}

fn bat_display_name(filename: &str) -> String {
    let stem = filename.trim_end_matches(".bat");
    if stem == "general" {
        return "GENERAL".to_string();
    }
    if let Some(rest) = stem.strip_prefix("general (") {
        if let Some(inner) = rest.strip_suffix(')') {
            return inner.to_string();
        }
    }
    stem.to_uppercase()
}

fn bat_to_id(filename: &str) -> String {
    let stem = filename.trim_end_matches(".bat");
    let mut id = stem.to_lowercase();
    if id == "general" {
        return "general".to_string();
    }
    if let Some(rest) = id.strip_prefix("general (") {
        if let Some(inner) = rest.strip_suffix(')') {
            id = inner.to_string();
        }
    }
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn infer_tags(filename: &str, content: &str) -> Vec<String> {
    let lower = format!("{filename} {content}").to_lowercase();
    let mut tags = Vec::new();

    if filename.to_uppercase().contains("APEX") || lower.contains("list-apex.txt") {
        tags.push("apex".to_string());
    }
    if lower.contains("discord") {
        tags.push("discord".to_string());
    }
    if lower.contains("youtube") || lower.contains("googlevideo") {
        tags.push("youtube".to_string());
    }
    if lower.contains("gamefilter") {
        tags.push("games".to_string());
    }
    if lower.contains("cloudflare")
        || filename.to_lowercase().starts_with("general")
    {
        tags.push("cloudflare".to_string());
    }

    tags.push("general".to_string());
    tags
}

pub fn game_filter_ports(root: &Path) -> (String, String, String) {
    let flag = root.join("utils").join("game_filter.enabled");
    if !flag.is_file() {
        return ("12".into(), "12".into(), "12".into());
    }
    let mode = std::fs::read_to_string(&flag)
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    match mode.as_str() {
        "all" => (
            "1024-65535".into(),
            "1024-65535".into(),
            "1024-65535".into(),
        ),
        "tcp" => ("1024-65535".into(), "1024-65535".into(), "12".into()),
        "udp" => ("1024-65535".into(), "12".into(), "1024-65535".into()),
        _ => ("12".into(), "12".into(), "12".into()),
    }
}

fn parse_winws_args(content: &str, root: &Path) -> Result<String, String> {
    let mut chunk = String::new();
    let mut capturing = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("::") || trimmed.starts_with("rem ") {
            continue;
        }

        if trimmed.contains("winws.exe") {
            capturing = true;
            if let Some(pos) = trimmed.find("winws.exe") {
                let after = trimmed[pos + "winws.exe".len()..].trim();
                let after = after.strip_prefix('"').unwrap_or(after);
                chunk.push_str(after);
            }
            continue;
        }

        if capturing {
            if trimmed == "^" || trimmed.ends_with('^') {
                let part = trimmed.trim_end_matches('^').trim();
                if !part.is_empty() {
                    chunk.push(' ');
                    chunk.push_str(part);
                }
                continue;
            }
            if trimmed.starts_with("--") {
                chunk.push(' ');
                chunk.push_str(trimmed.trim_end_matches('^').trim());
            } else if !trimmed.starts_with("start ") {
                break;
            }
        }
    }

    if chunk.is_empty() {
        return Err("В .bat не найдены аргументы winws.exe".to_string());
    }

    expand_bat_variables(&chunk, root)
}

fn expand_bat_variables(raw: &str, root: &Path) -> Result<String, String> {
    let bin = format!("{}\\", root.join("bin").display());
    let lists = format!("{}\\", root.join("lists").display());
    let (gf, gftcp, gfudp) = game_filter_ports(root);

    let mut s = raw.to_string();
    s = s.replace("%BIN%", &bin);
    s = s.replace("%LISTS%", &lists);
    s = s.replace("%GameFilterTCP%", &gftcp);
    s = s.replace("%GameFilterUDP%", &gfudp);
    s = s.replace("%GameFilter%", &gf);
    Ok(s)
}

/// Разбор строки аргументов winws (кавычки как в cmd).
pub fn split_command_line(line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

fn strategy_from_bat_path(path: &Path, root: &Path) -> Result<Strategy, String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Некорректное имя файла")?
        .to_string();
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Не удалось прочитать {}: {e}", path.display()))?;
    let args = parse_winws_args(&content, root).unwrap_or_default();
    let display_name = bat_display_name(&name);
    let id = bat_to_id(&name);
    let tags = infer_tags(&name, &content);
    let description = format!("Запуск: {}", name);
    Ok(Strategy {
        id,
        name: display_name,
        description,
        tags,
        source_bat: name,
        args,
    })
}

/// Сохранить пользовательский .bat в папку zapret (`general (ИМЯ).bat`).
pub fn save_custom_strategy(display_name: &str, content: &str) -> Result<Strategy, String> {
    let root = zapret_dir();
    if !root.join("bin").join("winws.exe").is_file() {
        return Err("Сначала установите zapret".into());
    }
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("Содержимое .bat пустое".into());
    }
    if !trimmed.to_lowercase().contains("winws.exe") {
        return Err("В файле должен быть вызов winws.exe".into());
    }

    let safe = sanitize_display_name(display_name);
    let filename = format!("general ({safe}).bat");
    let path = root.join(&filename);
    if path.is_file() {
        return Err(format!("Файл уже существует: {filename}"));
    }

    std::fs::write(&path, trimmed).map_err(|e| format!("Не удалось записать {filename}: {e}"))?;
    strategy_from_bat_path(&path, &root)
}

fn sanitize_display_name(name: &str) -> String {
    let t = name.trim();
    let base = if t.is_empty() { "CUSTOM" } else { t };
    let mut out = String::new();
    for c in base.chars().take(48) {
        match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => continue,
            _ => out.push(c),
        }
    }
    let out = out.trim().to_string();
    if out.is_empty() {
        "CUSTOM".to_string()
    } else {
        out
    }
}

/// Аргументы winws из .bat (без запуска `start`, без консоли).
pub fn winws_argv_from_bat(root: &Path, source_bat: &str) -> Result<Vec<String>, String> {
    let path = root.join(source_bat);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Не удалось прочитать {source_bat}: {e}"))?;
    let expanded = parse_winws_args(&content, root)?;
    Ok(split_command_line(&expanded))
}
