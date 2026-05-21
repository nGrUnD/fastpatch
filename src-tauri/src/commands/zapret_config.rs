use crate::commands::hosts::{backup_hosts_internal, hosts_path};
use crate::paths::zapret_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const HOSTS_BEGIN: &str = "# BEGIN ZAPRET (fastpatch)";
const HOSTS_END: &str = "# END ZAPRET (fastpatch)";

const IPSET_URL: &str =
    "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/refs/heads/main/.service/ipset-service.txt";
const HOSTS_URL: &str =
    "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/refs/heads/main/.service/hosts";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZapretSettings {
    pub game_filter: String,
    pub game_filter_label: String,
    pub ipset_mode: String,
    pub ipset_label: String,
    pub auto_update_check: bool,
}

fn utils_dir() -> PathBuf {
    zapret_dir().join("utils")
}

fn lists_dir() -> PathBuf {
    zapret_dir().join("lists")
}

fn read_game_filter() -> (String, String) {
    let flag = utils_dir().join("game_filter.enabled");
    if !flag.is_file() {
        return (
            "disabled".into(),
            "Выключен".into(),
        );
    }
    let mode = std::fs::read_to_string(&flag)
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let label = match mode.as_str() {
        "all" => "Включён (TCP и UDP)",
        "tcp" => "Включён (только TCP)",
        "udp" => "Включён (только UDP)",
        _ => "Включён",
    };
    (mode, label.to_string())
}

fn read_ipset_mode() -> (String, String) {
    let list = lists_dir().join("ipset-all.txt");
    if !list.is_file() {
        return ("any".into(), "Любой (файл отсутствует)".into());
    }
    let content = std::fs::read_to_string(&list).unwrap_or_default();
    let lines: Vec<_> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return ("any".into(), "Любой".into());
    }
    if lines.len() == 1 && lines[0].trim() == "203.0.113.113/32" {
        return ("none".into(), "Отключён".into());
    }
    ("loaded".into(), format!("Загружен ({} записей)", lines.len()))
}

fn read_auto_update() -> bool {
    utils_dir().join("check_updates.enabled").is_file()
}

#[tauri::command]
pub fn get_zapret_settings() -> ZapretSettings {
    let (game_filter, game_filter_label) = read_game_filter();
    let (ipset_mode, ipset_label) = read_ipset_mode();
    ZapretSettings {
        game_filter,
        game_filter_label,
        ipset_mode,
        ipset_label,
        auto_update_check: read_auto_update(),
    }
}

#[tauri::command]
pub fn set_game_filter(mode: String) -> Result<String, String> {
    std::fs::create_dir_all(utils_dir())
        .map_err(|e| format!("Не удалось создать utils: {e}"))?;
    let flag = utils_dir().join("game_filter.enabled");
    let mode = mode.to_lowercase();
    match mode.as_str() {
        "disabled" | "off" | "0" => {
            if flag.exists() {
                std::fs::remove_file(&flag).map_err(|e| e.to_string())?;
            }
            Ok("Игровой фильтр выключен. Перезапустите стратегию.".into())
        }
        "all" | "tcp" | "udp" => {
            std::fs::write(&flag, mode).map_err(|e| e.to_string())?;
            Ok("Игровой фильтр обновлён. Перезапустите активную стратегию.".into())
        }
        _ => Err("Режим: disabled, all, tcp или udp".into()),
    }
}

#[tauri::command]
pub fn set_ipset_mode(mode: String) -> Result<String, String> {
    let list = lists_dir().join("ipset-all.txt");
    let backup = lists_dir().join("ipset-all.txt.backup");
    std::fs::create_dir_all(lists_dir()).map_err(|e| e.to_string())?;

    match mode.to_lowercase().as_str() {
        "none" => {
            if list.exists() && !backup.exists() {
                std::fs::rename(&list, &backup).map_err(|e| e.to_string())?;
            }
            std::fs::write(&list, "203.0.113.113/32\n").map_err(|e| e.to_string())?;
            Ok("IPSet фильтр отключён".into())
        }
        "any" => {
            std::fs::write(&list, "").map_err(|e| e.to_string())?;
            Ok("IPSet: режим «любой»".into())
        }
        "loaded" => {
            if backup.is_file() {
                if list.exists() {
                    std::fs::remove_file(&list).ok();
                }
                std::fs::rename(&backup, &list).map_err(|e| e.to_string())?;
                Ok("IPSet восстановлен из резервной копии".into())
            } else {
                Err("Нет резервной копии. Сначала обновите список IPSet.".into())
            }
        }
        _ => Err("Режим: none, any или loaded".into()),
    }
}

#[tauri::command]
pub fn set_auto_update_check(enabled: bool) -> Result<String, String> {
    std::fs::create_dir_all(utils_dir()).map_err(|e| e.to_string())?;
    let flag = utils_dir().join("check_updates.enabled");
    if enabled {
        std::fs::write(&flag, "ENABLED\n").map_err(|e| e.to_string())?;
        Ok("Автопроверка обновлений включена".into())
    } else if flag.exists() {
        std::fs::remove_file(&flag).map_err(|e| e.to_string())?;
        Ok("Автопроверка обновлений выключена".into())
    } else {
        Ok("Автопроверка уже выключена".into())
    }
}

async fn download_text(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("fastpatch/0.1")
        .timeout(std::time::Duration::from_secs(120))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Ошибка загрузки: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Сервер вернул {} для {}",
            response.status(),
            url
        ));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Не удалось прочитать ответ: {e}"))
}

async fn download_to_file(url: &str, path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = download_text(url).await?;
    std::fs::write(path, &text).map_err(|e| format!("Ошибка записи {}: {e}", path.display()))?;
    Ok(())
}

fn zapret_hosts_marker_lines(remote: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = remote
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    if lines.is_empty() {
        return None;
    }
    Some((lines[0].to_string(), lines[lines.len() - 1].to_string()))
}

fn merge_zapret_hosts_into_system(current: &str, remote: &str) -> Result<(String, &'static str), String> {
    let remote = remote.trim();
    if remote.is_empty() {
        return Err("Файл hosts с репозитория пуст".into());
    }

    if current.contains(HOSTS_BEGIN) && current.contains(HOSTS_END) {
        let before = current
            .split(HOSTS_BEGIN)
            .next()
            .unwrap_or("")
            .trim_end();
        let after = current
            .split(HOSTS_END)
            .nth(1)
            .unwrap_or("")
            .trim_start();
        let mut out = String::new();
        if !before.is_empty() {
            out.push_str(before);
            if !before.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }
        out.push_str(HOSTS_BEGIN);
        out.push('\n');
        out.push_str(remote);
        if !remote.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(HOSTS_END);
        if !after.is_empty() {
            out.push('\n');
            out.push_str(after);
            if !after.ends_with('\n') {
                out.push('\n');
            }
        }
        return Ok((out, "обновлён"));
    }

    if let Some((first, last)) = zapret_hosts_marker_lines(remote) {
        if current.contains(&first) && current.contains(&last) {
            return Ok((current.to_string(), "уже актуален"));
        }
    }

    let mut out = current.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str(HOSTS_BEGIN);
    out.push('\n');
    out.push_str(remote);
    if !remote.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(HOSTS_END);
    out.push('\n');

    Ok((out, "добавлен"))
}

fn count_host_entries(text: &str) -> usize {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .count()
}

#[tauri::command]
pub async fn update_ipset_list() -> Result<String, String> {
    let path = lists_dir().join("ipset-all.txt");
    download_to_file(IPSET_URL, &path).await?;
    Ok("Список IPSet обновлён".into())
}

#[tauri::command]
pub async fn update_zapret_hosts_file() -> Result<String, String> {
    #[cfg(windows)]
    {
        if !crate::win_process::is_elevated() {
            return Err(
                "Нужны права администратора для записи C:\\Windows\\System32\\drivers\\etc\\hosts"
                    .into(),
            );
        }
    }

    let remote = download_text(HOSTS_URL).await?;
    let entries = count_host_entries(&remote);

    let path = hosts_path();
    let current = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "Не удалось прочитать {} (запустите fastpatch от администратора): {e}",
            path.display()
        )
    })?;

    let (merged, action) = merge_zapret_hosts_into_system(&current, &remote)?;

    if action == "уже актуален" {
        return Ok(format!(
            "Системный hosts уже содержит записи zapret ({entries} строк в репозитории)."
        ));
    }

    let backup_name = backup_hosts_internal()?;
    std::fs::write(&path, &merged).map_err(|e| {
        format!(
            "Не удалось записать hosts ({}): {e}",
            path.display()
        )
    })?;

    Ok(format!(
        "Hosts zapret {action}: {entries} записей. Резервная копия: {backup_name}. Откройте раздел Hosts для просмотра."
    ))
}
