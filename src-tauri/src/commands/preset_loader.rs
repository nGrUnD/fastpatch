//! Пресеты Zapret 2 (`presets/*.txt`).

use super::Strategy;
use crate::commands::apex;
use crate::paths::zapret2_dir;

/// Имена пресетов (slug), которые проверяем первыми при автоподборе Apex.
pub const APEX_PRESET_ID: &str = "apex-legends";

/// Имена пресетов (slug), которые проверяем первыми при автоподборе.
const AUTODETECT_PRIORITY: &[&str] = &[
    "default-v5",
    "default-v4",
    "default-v3",
    "default-v2",
    "default",
    "general",
    "discord-youtube",
    "youtube-discord",
    "yandex",
    "rostelecom",
    "megafon",
    "mts",
    "beeline",
    "tele2",
];

fn preset_name_to_id(name: &str) -> String {
    let lower = name.to_lowercase();
    lower
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c == ' ' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn preset_rel_path(filename: &str) -> String {
    format!("presets/{filename}")
}

fn push_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.iter().any(|t| t == tag) {
        tags.push(tag.to_string());
    }
}

fn preset_tags(display_name: &str) -> Vec<String> {
    let lower = display_name.to_lowercase();
    let mut tags = vec!["zapret2".to_string(), "preset".to_string()];

    if lower.contains("default v5") {
        push_tag(&mut tags, "recommended");
    }
    if lower.contains("apex") {
        push_tag(&mut tags, "apex");
        push_tag(&mut tags, "games");
        push_tag(&mut tags, "discord");
        push_tag(&mut tags, "youtube");
        push_tag(&mut tags, "recommended");
    }
    if lower.contains("discord") || lower.contains("default") || lower.contains("general") {
        push_tag(&mut tags, "discord");
    }
    if lower.contains("youtube") || lower.contains("default") || lower.contains("general") {
        push_tag(&mut tags, "youtube");
    }
    if lower.contains("game filter")
        || lower.contains("gaming")
        || lower.contains("dead by daylight")
        || lower.contains("valorant")
        || lower.contains("riot")
    {
        push_tag(&mut tags, "games");
    }
    if lower.contains("all tcp") || lower.contains("complex") || lower.contains("multidisorder") {
        push_tag(&mut tags, "aggressive");
    }
    if lower.contains("old") || lower.contains("legacy") || lower.contains("general alt") {
        push_tag(&mut tags, "legacy");
    }
    if lower.contains("ростелеком")
        || lower.contains("mts")
        || lower.contains("megafon")
        || lower.contains("beeline")
        || lower.contains("tele2")
        || lower.contains("rostelecom")
    {
        push_tag(&mut tags, "provider");
    }
    if lower.contains("preset x") || lower.contains("preset xy") {
        push_tag(&mut tags, "experimental");
    }

    tags
}

fn preset_description(display_name: &str, tags: &[String]) -> String {
    let lower = display_name.to_lowercase();
    if tags.iter().any(|t| t == "apex") {
        return "Apex Legends: Discord/YouTube + EA web; EAC/лобби/игровой UDP без DPI".to_string();
    }
    if lower.contains("default v5") {
        return "Рекомендуемый универсальный пресет для Discord/YouTube. Для Apex используйте отдельный профиль.".to_string();
    }
    if tags.iter().any(|t| t == "aggressive") {
        return "Агрессивный профиль для сложных провайдеров; может сильнее влиять на игры и задержку.".to_string();
    }
    if tags.iter().any(|t| t == "games") {
        return "Игровой профиль с game filter; тестируйте в конкретной игре.".to_string();
    }
    if tags.iter().any(|t| t == "provider") {
        return "Профиль под конкретного провайдера.".to_string();
    }
    if tags.iter().any(|t| t == "legacy") {
        return "Legacy/ALT-вариант для ручного подбора, если Default не сработал.".to_string();
    }
    "Пресет Zapret 2 (winws2)".to_string()
}

pub fn load_presets() -> Result<Vec<Strategy>, String> {
    let _ = apex::ensure_apex_assets_v2();

    let root = zapret2_dir();
    let presets_dir = root.join("presets");
    if !presets_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut strategies = Vec::new();
    let entries = std::fs::read_dir(&presets_dir)
        .map_err(|e| format!("Не удалось прочитать presets/: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if filename.starts_with('_') {
            continue;
        }

        let display_name = filename.trim_end_matches(".txt").to_string();
        let id = preset_name_to_id(&display_name);
        let rel = preset_rel_path(&filename);
        let tags = preset_tags(&display_name);
        let description = preset_description(&display_name, &tags);

        strategies.push(Strategy {
            id,
            name: display_name,
            description,
            tags,
            source_bat: rel,
            args: String::new(),
        });
    }

    strategies.sort_by(|a, b| preset_sort_key(&a.id).cmp(&preset_sort_key(&b.id)));

    Ok(strategies)
}

fn preset_sort_key(id: &str) -> (u8, u32, String) {
    if id == APEX_PRESET_ID {
        return (0, 0, String::new());
    }
    if let Some(pos) = AUTODETECT_PRIORITY.iter().position(|p| *p == id) {
        return (0, pos as u32 + 1, String::new());
    }
    (1, 0, id.to_string())
}

pub fn autodetect_priority(id: &str) -> (u8, u32, String) {
    preset_sort_key(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_name_to_id_normalizes_spaces_and_symbols() {
        assert_eq!(preset_name_to_id("Apex Legends"), "apex-legends");
        assert_eq!(
            preset_name_to_id("ALL TCP & UDP multisplit_sni"),
            "all-tcp-udp-multisplit-sni"
        );
    }

    #[test]
    fn apex_preset_sorts_before_default_presets() {
        assert!(autodetect_priority(APEX_PRESET_ID) < autodetect_priority("default-v5"));
    }

    #[test]
    fn preset_tags_classify_common_presets() {
        let default_tags = preset_tags("Default v5");
        assert!(default_tags.iter().any(|t| t == "recommended"));
        assert!(default_tags.iter().any(|t| t == "discord"));
        assert!(default_tags.iter().any(|t| t == "youtube"));

        let aggressive_tags = preset_tags("ALL TCP & UDP multisplit_sni");
        assert!(aggressive_tags.iter().any(|t| t == "aggressive"));

        let apex_tags = preset_tags("Apex Legends");
        assert!(apex_tags.iter().any(|t| t == "apex"));
        assert!(apex_tags.iter().any(|t| t == "games"));
    }
}
